//! Code for the [`FlatMatcher`].

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Build a function with a single match statement to quickly map byte sequences
/// to values.
///
/// The generated function will accept a slice of bytes (`&[u8]`) and will
/// return a tuple containing:
///
///   1. The match, if any (`Option<{return_type}>`)
///   2. Either, depending on [`Self::return_slice()`]:
///      * The remainder of the slice (`&[u8]`).
///      * The index of the next unmatched byte (`usize`).
///
/// See [`Self::return_index()`] if you need a `const fn` matcher.
///
/// For example, suppose you generate a [matcher for all HTML entities][htmlize]
/// called `entity_matcher()`:
///
/// ```rust,ignore
/// assert!(entity_matcher(b"&times;XY") == (Some("×"), b"XY".as_slice()));
/// ```
///
///   * The prefixes it checks do not all have to be the same length.
///   * If more than one prefix matches, it will return the longest one.
///   * If nothing matches, it will return `(None, &input)`.
///
/// Since the matchers only check the start of the input, you will want to
/// use [`iter().position()`] or the [memchr crate][memchr] to find the
/// start of a potential match.
///
/// # Example build script
///
/// ```rust
/// use matchgen::FlatMatcher;
/// use std::env;
/// use std::error::Error;
/// use std::fs::File;
/// use std::io::{BufWriter, Read, Write};
/// use std::path::Path;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     # let tmp_dir = temp_dir::TempDir::new().unwrap();
///     # env::set_var("OUT_DIR", tmp_dir.path());
///     let out_path = Path::new(&env::var("OUT_DIR")?).join("matcher.rs");
///     let mut out = BufWriter::new(File::create(out_path)?);
///
///     writeln!(out, "/// My fancy matcher.")?;
///     FlatMatcher::new("pub fn fancy_matcher", "&'static [u8]")
///         .add(b"one", r#"b"1""#)
///         .add(b"two", r#"b"2""#)
///         .add(b"three", r#"b"3""#)
///         .render(&mut out)?;
///
///     Ok(())
/// }
/// ```
///
/// To use the matcher:
///
/// ```rust,ignore
/// include!(concat!(env!("OUT_DIR"), "/matcher.rs"));
///
/// fn main() {
///     assert_eq!(
///         fancy_matcher(b"one two three"),
///         (Some(b"1"), b" two three".as_slice()),
///     );
/// }
/// ```
///
/// [build script]: https://doc.rust-lang.org/cargo/reference/build-scripts.html
/// [`iter().position()`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.position
/// [memchr]: http://docs.rs/memchr
/// [htmlize]: https://crates.io/crates/htmlize
#[derive(Clone, Debug)]
pub struct FlatMatcher {
    /// The first part of the function definition to generate, e.g.
    /// `"pub fn matcher"`.
    pub fn_name: String,

    /// The return type (will be wrapped in [`Option`]), e.g. `"&'static str"`.
    pub return_type: String,

    /// Whether to return the remainder as a slice or an index.
    ///
    /// If `false`, the remainder will be returned as the index of the next
    /// unmatched byte (which might be past the end of the input), and the
    /// generated matching function will be `const`.
    ///
    /// Defaults to `true`.
    pub return_slice: bool,

    /// Whether to prevent Clippy from evaluating the generated code. Defaults
    /// to `false`.
    ///
    /// See [`Self::disable_clippy()`].
    pub disable_clippy: bool,

    /// Whether to mark the function with [`#[must_use]`][must_use]. Defaults to
    /// `true`.
    ///
    /// [must_use]: https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-must_use-attribute
    pub must_use: bool,

    /// Doc attribute, e.g. `#[doc = "Documenation"]`, to add to the function.
    ///
    /// Should not have a trailing newline.
    pub doc: Option<String>,

    /// The arms of the match statement.
    pub arms: HashMap<Vec<u8>, String>,
}

impl FlatMatcher {
    /// Create a new matcher (for use in a build script).
    ///
    /// This will generate a matcher with the the specified function name and
    /// return type. You can add matches to it with [`Self::add()`] and/or
    /// [`Self::extend()`], then turn it into code with [`Self::render()`].
    ///
    /// See the [struct documentation][FlatMatcher] for a complete example.
    #[allow(clippy::needless_pass_by_value)] // ToString can borrow.
    pub fn new<N, R>(fn_name: N, return_type: R) -> Self
    where
        N: ToString,
        R: ToString,
    {
        Self {
            fn_name: fn_name.to_string(),
            return_type: return_type.to_string(),
            return_slice: true,
            disable_clippy: false,
            must_use: true,
            doc: None,
            arms: HashMap::default(),
        }
    }

    /// Add a match.
    ///
    /// ```rust
    /// let mut matcher = matchgen::FlatMatcher::new("fn matcher", "u64");
    /// matcher.add(b"a", "1");
    /// ```
    pub fn add<'a, K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: IntoIterator<Item = &'a u8>,
        V: Into<String>,
    {
        self.arms
            .insert(key.into_iter().copied().collect(), value.into());
        self
    }

    /// Set the function to return the remainder as a slice.
    ///
    /// That is, the signature of the generated will look something like:
    ///
    /// ```rust,ignore
    /// fn matcher(input: &[u8]) -> (Option<ReturnType>, &[u8]) {
    /// #     (ReturnType, input)
    /// # }
    /// ```
    ///
    /// This is the opposite of [`Self::return_index()`].
    pub fn return_slice(&mut self) -> &mut Self {
        self.return_slice = true;
        self
    }

    /// Set the function to return the remainder as an index.
    ///
    /// That is, the signature of the generated will look something like:
    ///
    /// ```rust,ignore
    /// const fn matcher(input: &[u8]) -> (Option<ReturnType>, usize) {
    /// #     (ReturnType, input)
    /// # }
    /// ```
    ///
    /// This is the opposite of [`Self::return_slice()`].
    ///
    /// The returned index might be just past the end of the input if the entire
    /// input was matched.
    ///
    /// This will cause the generated matching function to be `const`, since
    /// it won’t require any `const` incompatible features.
    pub fn return_index(&mut self) -> &mut Self {
        self.return_slice = false;
        self
    }

    /// Set whether or not to prevent [Clippy] from evaluating the generated
    /// code.
    ///
    /// If set to `true`, this will use conditional compilation to prevent
    /// [Clippy] from evaluating the generated code, and will provide a stub
    /// function to ensure that the Clippy pass builds.
    ///
    /// This may be useful if the produced matcher is particularly big. I don’t
    /// recommend setting this to `true` unless you notice a delay when running
    /// `cargo clippy`.
    ///
    /// [Clippy]: https://doc.rust-lang.org/clippy/
    pub fn disable_clippy(&mut self, disable: bool) -> &mut Self {
        self.disable_clippy = disable;
        self
    }

    /// Set whether or not to mark the generated function with [`#[must_use]`].
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut out = Vec::new();
    /// matchgen::FlatMatcher::new("fn match_bytes", "u64")
    ///     .must_use(false)
    ///     .add("a".as_bytes(), "1")
    ///     .render(&mut out)
    ///     .unwrap();
    ///
    /// use bstr::ByteVec;
    /// pretty_assertions::assert_str_eq!(
    ///     r#"fn match_bytes(slice: &[u8]) -> (Option<u64>, &[u8]) {
    ///     #[allow(unreachable_patterns)]
    ///     match slice {
    ///         [97, ..] => (Some(1), &slice[1..]),
    ///         _ => (None, slice),
    ///     }
    /// }
    /// "#,
    ///     out.into_string().unwrap(),
    /// );
    /// ```
    pub fn must_use(&mut self, must_use: bool) -> &mut Self {
        self.must_use = must_use;
        self
    }

    /// Don’t include documentation for the matcher.
    ///
    /// This is the default behavior.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut out = Vec::new();
    /// matchgen::FlatMatcher::new("fn match_bytes", "u64")
    ///     .remove_doc()
    ///     .add("a".as_bytes(), "1")
    ///     .render(&mut out)
    ///     .unwrap();
    ///
    /// use bstr::ByteVec;
    /// pretty_assertions::assert_str_eq!(
    ///     r#"#[must_use]
    /// fn match_bytes(slice: &[u8]) -> (Option<u64>, &[u8]) {
    ///     #[allow(unreachable_patterns)]
    ///     match slice {
    ///         [97, ..] => (Some(1), &slice[1..]),
    ///         _ => (None, slice),
    ///     }
    /// }
    /// "#,
    ///     out.into_string().unwrap(),
    /// );
    /// ```
    pub fn remove_doc(&mut self) -> &mut Self {
        self.doc = None;
        self
    }

    /// Set documentation for the matcher to a string.
    ///
    /// This is normally what you want. Just pass a string with the
    /// documentation for your matcher, and it will produce an [attribute][]
    /// like `#[doc = "My function documentation..."]`.
    ///
    /// The `doc` argument should produce a Rust string literal when rendered
    /// with [`fmt::Debug`]. A normal [`String`] or [`str`] will work.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut out = Vec::new();
    /// matchgen::FlatMatcher::new("fn match_bytes", "u64")
    ///     .doc(
    ///         "Match the first bytes of a slice.
    ///
    ///         Matches only the string `\"a\"` and returns `1`."
    ///     )
    ///     .add("a".as_bytes(), "1")
    ///     .render(&mut out)
    ///     .unwrap();
    ///
    /// use bstr::ByteVec;
    /// pretty_assertions::assert_str_eq!(
    ///     r#"#[doc = "Match the first bytes of a slice.\n\n        Matches only the string `\"a\"` and returns `1`."]
    /// #[must_use]
    /// fn match_bytes(slice: &[u8]) -> (Option<u64>, &[u8]) {
    ///     #[allow(unreachable_patterns)]
    ///     match slice {
    ///         [97, ..] => (Some(1), &slice[1..]),
    ///         _ => (None, slice),
    ///     }
    /// }
    /// "#,
    ///     out.into_string().unwrap(),
    /// );
    /// ```
    ///
    /// [attribute]: https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html
    pub fn doc<S: fmt::Debug>(&mut self, doc: S) -> &mut Self {
        self.doc = Some(format!("#[doc = {:?}]", doc));
        self
    }

    /// Set documentation for the matcher to a Rust expression.
    ///
    /// Generally you want [`Self::doc()`], not this. This can produce an
    /// [attribute][] like `#[doc = include_str!("my_func.md")]`.
    ///
    /// The `doc` argument should produce a Rust expression when rendered with
    /// [`fmt::Display`]. A normal [`String`] or [`str`] will work.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut out = Vec::new();
    /// matchgen::FlatMatcher::new("fn match_bytes", "u64")
    ///     .doc_raw(r#"include_str!("match_bytes.md")"#)
    ///     .add("a".as_bytes(), "1")
    ///     .render(&mut out)
    ///     .unwrap();
    ///
    /// use bstr::ByteVec;
    /// pretty_assertions::assert_str_eq!(
    ///     r#"#[doc = include_str!("match_bytes.md")]
    /// #[must_use]
    /// fn match_bytes(slice: &[u8]) -> (Option<u64>, &[u8]) {
    ///     #[allow(unreachable_patterns)]
    ///     match slice {
    ///         [97, ..] => (Some(1), &slice[1..]),
    ///         _ => (None, slice),
    ///     }
    /// }
    /// "#,
    ///     out.into_string().unwrap(),
    /// );
    /// ```
    ///
    /// [attribute]: https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html
    pub fn doc_raw<S: fmt::Display>(&mut self, doc: S) -> &mut Self {
        self.doc = Some(format!("#[doc = {}]", doc));
        self
    }

    /// Set documentation for the matcher to an option.
    ///
    /// Generally you want [`Self::doc()`], not this. This can produce an
    /// [attribute][] like `#[doc(hidden)]`.
    ///
    /// The `doc` argument should produce the options when rendered with
    /// [`fmt::Display`]. A normal [`String`] or [`str`] will work.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut out = Vec::new();
    /// matchgen::FlatMatcher::new("fn match_bytes", "u64")
    ///     .doc_option("hidden")
    ///     .render(&mut out)
    ///     .unwrap();
    ///
    /// use bstr::ByteVec;
    /// pretty_assertions::assert_str_eq!(
    ///     r#"#[doc(hidden)]
    /// #[must_use]
    /// fn match_bytes(slice: &[u8]) -> (Option<u64>, &[u8]) {
    ///     #[allow(unreachable_patterns)]
    ///     match slice {
    ///         _ => (None, slice),
    ///     }
    /// }
    /// "#,
    ///     out.into_string().unwrap(),
    /// );
    /// ```
    ///
    /// [attribute]: https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html
    pub fn doc_option<S: fmt::Display>(&mut self, doc: S) -> &mut Self {
        self.doc = Some(format!("#[doc({})]", doc));
        self
    }

    /// Write the matcher as a Rust source file in `$OUT_DIR`.
    ///
    /// This is what you want if you’re using this in `build.rs` as intended.
    /// See the [`FlatMatcher`] for a full example.
    ///
    /// This will overwrite the file if it already exists, or create a new file
    /// if it does not.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::env::var;
    /// use std::path::Path;
    /// use std::fs;
    ///
    /// # let tmp_dir = temp_dir::TempDir::new().unwrap();
    /// # std::env::set_var("OUT_DIR", tmp_dir.path());
    /// matchgen::FlatMatcher::new("fn match_bytes", "u64")
    ///     .add("a".as_bytes(), "1")
    ///     .write_to_out_dir("matcher.rs")
    ///     .unwrap();
    ///
    /// let path = Path::new(&var("OUT_DIR").unwrap()).join("matcher.rs");
    /// assert!(fs::read_to_string(path).unwrap().contains("fn match_bytes"));
    /// ```
    ///
    /// # Errors
    ///
    /// This can return [`io::Error`] if there is a problem writing the file, or
    /// if `$OUT_DIR` isn’t set to a UTF-8 string.
    pub fn write_to_out_dir<P: AsRef<Path>>(
        &self,
        sub_path: P,
    ) -> io::Result<()> {
        let out_dir = &env::var("OUT_DIR")
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        self.write_to_path(Path::new(out_dir).join(sub_path))
    }

    /// Write the matcher as a Rust source file at `path`.
    ///
    /// This will overwrite the file if it already exists, or create a new file
    /// if it does not.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let tmp_dir = temp_dir::TempDir::new().unwrap();
    /// # let path = tmp_dir.child("test.rs");
    /// matchgen::FlatMatcher::new("fn match_bytes", "u64")
    ///     .add("a".as_bytes(), "1")
    ///     .write_to_path(&path)
    ///     .unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// This can return [`io::Error`] if there is a problem writing to `path`.
    pub fn write_to_path<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut out = io::BufWriter::new(fs::File::create(path)?);
        self.render(&mut out)?;
        out.flush()
    }

    /// Render the matcher into Rust code.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use bstr::ByteVec;
    /// use matchgen::FlatMatcher;
    /// use pretty_assertions::assert_str_eq;
    ///
    /// let mut out = Vec::new();
    /// let mut matcher = FlatMatcher::new("fn match_bytes", "u64");
    /// matcher.disable_clippy(true);
    /// matcher.extend([("a".as_bytes(), "1")]);
    /// matcher.render(&mut out).unwrap();
    ///
    /// assert_str_eq!(
    ///     r#"#[cfg(not(clippy))]
    /// #[must_use]
    /// fn match_bytes(slice: &[u8]) -> (Option<u64>, &[u8]) {
    ///     #[allow(unreachable_patterns)]
    ///     match slice {
    ///         [97, ..] => (Some(1), &slice[1..]),
    ///         _ => (None, slice),
    ///     }
    /// }
    ///
    /// #[cfg(clippy)]
    /// #[must_use]
    /// fn match_bytes(slice: &[u8]) -> (Option<u64>, &[u8]) {
    ///     (None, slice)
    /// }
    /// "#,
    ///     out.into_string().unwrap(),
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// This can return [`io::Error`] if there is a problem writing to `writer`.
    pub fn render<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.disable_clippy {
            writeln!(writer, "#[cfg(not(clippy))]")?;
        }

        self.render_func(writer)?;

        if self.disable_clippy {
            writeln!(writer)?;
            writeln!(writer, "#[cfg(clippy)]")?;
            self.render_stub(writer)?;
        }

        Ok(())
    }

    /// Render the function that does the matching.
    ///
    /// Generally you should use [`Self::render()`] instead of this.
    ///
    /// # Errors
    ///
    /// This can return [`io::Error`] if there is a problem writing to `writer`.
    pub fn render_func<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let indent = "    "; // Our formatting prevents embedding this.

        self.render_fn_start(writer, "slice")?;

        write!(
            writer,
            "{indent}#[allow(unreachable_patterns)]\n\
            {indent}match slice {{\n",
            indent = indent,
        )?;

        // Output entries in longest to shortest order.
        let mut entries: Vec<_> = self.arms.iter().collect();
        entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        for (key, value) in entries {
            let count = key.len();
            writeln!(
                writer,
                "{indent}    [{prefix}..] => (Some({value}), {remainder}),",
                indent = indent,
                prefix = if count == 0 {
                    String::new()
                } else {
                    format!("{}, ", itertools::join(key.iter(), ", "))
                },
                value = value,
                remainder = if self.return_slice {
                    format!("&slice[{}..]", count)
                } else {
                    count.to_string()
                },
            )?;
        }

        write!(
            writer,
            "{indent}    _ => (None, {remainder}),\n\
            {indent}}}\n\
            }}\n",
            indent = indent,
            remainder = if self.return_slice { "slice" } else { "0" },
        )?;

        Ok(())
    }

    /// Render a stub that does nothing.
    ///
    /// Generally you should use [`Self::render()`] instead of this.
    ///
    /// # Errors
    ///
    /// This can return [`io::Error`] if there is a problem writing to `writer`.
    pub fn render_stub<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let indent = "    "; // Our formatting prevents embedding this.

        self.render_fn_start(
            writer,
            if self.return_slice { "slice" } else { "_" },
        )?;

        write!(
            writer,
            "{indent}(None, {remainder})\n\
            }}\n",
            indent = indent,
            remainder = if self.return_slice { "slice" } else { "0" },
        )?;

        Ok(())
    }

    /// Get the function name with it modifiers.
    ///
    /// This will add `const` in the appropriate place if necessary.
    fn fn_name(&self) -> String {
        if self.return_slice {
            self.fn_name.clone()
        } else if self.fn_name.starts_with("fn ") {
            format!("const {}", self.fn_name)
        } else {
            // FIXME what if a tab or other non-space is used?
            self.fn_name.replace(" fn ", " const fn ")
        }
    }

    /// Render attributes for the function or stub.
    ///
    /// # Errors
    ///
    /// This can return [`io::Error`] if there is a problem writing to `writer`.
    fn render_fn_start<W: io::Write>(
        &self,
        writer: &mut W,
        parameter: &str,
    ) -> io::Result<()> {
        if let Some(doc) = &self.doc {
            writeln!(writer, "{}", doc)?;
        }

        if self.must_use {
            writeln!(writer, "#[must_use]")?;
        }

        writeln!(
            writer,
            "{fn_name}({parameter}: &[u8]) -> (Option<{return_type}>, {remainder_type}) {{",
            fn_name = self.fn_name(),
            parameter = parameter,
            return_type = &self.return_type,
            remainder_type = if self.return_slice { "&[u8]" } else { "usize" },
        )?;

        Ok(())
    }
}

impl<'a, K, V> Extend<(K, V)> for FlatMatcher
where
    K: IntoIterator<Item = &'a u8>,
    V: Into<String>,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        iter.into_iter().for_each(|(key, value)| {
            self.add(key, value);
        });
    }
}

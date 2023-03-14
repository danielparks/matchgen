use std::collections::HashMap;
use std::io;

/// Build a function with a single match statement to quickly map byte sequences
/// to values.
///
/// The generated function will accept a slice of bytes (`&[u8]`) and will
/// return a tuple containing the match, if any, and the remainder of the slice,
/// i.e. `(Option<{return_type}>, &[u8])`.
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

    /// Whether or not to prevent Clippy from evaluating the generated code.
    ///
    /// See [`Self::disable_clippy()`].
    pub disable_clippy: bool,

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
    pub fn new<N: AsRef<str>, R: AsRef<str>>(
        fn_name: N,
        return_type: R,
    ) -> Self {
        Self {
            fn_name: fn_name.as_ref().to_string(),
            return_type: return_type.as_ref().to_string(),
            disable_clippy: false,
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
    ///     r#"#[cfg(not(feature = "cargo-clippy"))]
    /// fn match_bytes(slice: &[u8]) -> (Option<u64>, &[u8]) {
    ///     #[allow(unreachable_patterns)]
    ///     match slice {
    ///         [97, ..] => (Some(1), &slice[1..]),
    ///         _ => (None, slice),
    ///     }
    /// }
    ///
    /// #[cfg(feature = "cargo-clippy")]
    /// fn match_bytes(slice: &[u8]) -> (Option<u64>, &[u8]) {
    ///     (None, slice)
    /// }
    /// "#,
    ///     out.into_string().unwrap(),
    /// );
    /// ```
    pub fn render<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.disable_clippy {
            writeln!(writer, "#[cfg(not(feature = \"cargo-clippy\"))]")?;
        }

        self.render_func(writer)?;

        if self.disable_clippy {
            writeln!(writer)?;
            writeln!(writer, "#[cfg(feature = \"cargo-clippy\")]")?;
            self.render_stub(writer)?;
        }

        Ok(())
    }

    /// Render the function that does the matching.
    ///
    /// Generally you should use [`Self::render()`] instead of this.
    pub fn render_func<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let indent = "    "; // Our formatting prevents embedding this.
        let fn_name = &self.fn_name;
        let return_type = &self.return_type;

        write!(
            writer,
            "{fn_name}(slice: &[u8]) -> (Option<{return_type}>, &[u8]) {{\n\
            {indent}#[allow(unreachable_patterns)]\n\
            {indent}match slice {{\n",
        )?;

        // Output entries in longest to shortest order.
        let mut entries: Vec<_> = self.arms.iter().collect();
        entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        for (key, value) in entries {
            let count = key.len();
            if count == 0 {
                writeln!(
                    writer,
                    "{indent}    [..] => (Some({value}), slice),",
                )?;
            } else {
                let chars = itertools::join(key.iter(), ", ");
                writeln!(
                    writer,
                    "{indent}    [{chars}, ..] => (Some({value}), &slice[{count}..]),",
                )?;
            }
        }

        write!(
            writer,
            "{indent}    _ => (None, slice),\n\
            {indent}}}\n\
            }}\n"
        )?;

        Ok(())
    }

    /// Render a stub that does nothing.
    ///
    /// Generally you should use [`Self::render()`] instead of this.
    pub fn render_stub<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let indent = "    "; // Our formatting prevents embedding this.
        let fn_name = &self.fn_name;
        let return_type = &self.return_type;

        write!(
            writer,
            "{fn_name}(slice: &[u8]) -> (Option<{return_type}>, &[u8]) {{\n\
            {indent}(None, slice)\n\
            }}\n"
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

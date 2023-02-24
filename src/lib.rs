//! [`IterMatcher`] can be used from a [build script] to generate a function
//! that accepts an iterator over bytes and returns a mapped value if it finds a
//! given byte sequence at the start of the iterator.
//!
//! # Example build script
//!
//! ```rust
//! use iter_matcher::IterMatcher;
//! use std::env;
//! use std::error::Error;
//! use std::fs::File;
//! use std::io::{BufWriter, Read, Write};
//! use std::path::Path;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     # let tmp_dir = temp_dir::TempDir::new().unwrap();
//!     # env::set_var("OUT_DIR", tmp_dir.path());
//!     let out_path = Path::new(&env::var("OUT_DIR")?).join("matcher.rs");
//!     let mut out = BufWriter::new(File::create(out_path)?);
//!
//!     writeln!(out, "/// My fancy matcher.")?;
//!     IterMatcher::new("pub fn fancy_matcher", "&'static [u8]")
//!         .add(b"one", r#"b"1""#)
//!         .add(b"two", r#"b"2""#)
//!         .add(b"three", r#"b"3""#)
//!         .render_iter(&mut out)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! To use the matcher:
//!
//! ```rust,ignore
//! include!(concat!(env!("OUT_DIR"), "/matcher.rs"));
//!
//! fn main() {
//!     let mut iter = b"one two three".iter();
//!     assert!(fancy_matcher(&mut iter) == Some(b"1"));
//!     assert!(iter.as_slice() == b" two three");
//! }
//! ```
//!
//! # Minimum supported Rust version
//!
//! Currently the minimum supported Rust version (MSRV) is **1.60**.
//!
//! [build script]: https://doc.rust-lang.org/cargo/reference/build-scripts.html

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::io;

/// A node in a matcher’s simple finite-state automaton.
///
/// You probably want to use [`IterMatcher`] instead.
#[derive(Debug, Default)]
pub struct Node {
    /// If the matcher gets to this node and `leaf` is `Some(_)`, then we found
    /// a (potential) match.
    ///
    /// If `branch` is not empty, then we still need to check further on to see
    /// if there’s a longer match.
    pub leaf: Option<String>,

    /// The list of characters that could be matched next, and the nodes they
    /// represent.
    ///
    /// If none of these characters match, then return `leaf` as the match
    /// (it might be None, indicating that nothing matches).
    pub branch: HashMap<u8, Node>,
}

impl Node {
    /// Add a match rooted in this node.
    ///
    /// ```rust
    /// let mut node = iter_matcher::Node::default();
    /// node.add(b"a", "1");
    /// ```
    pub fn add<'a, K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: IntoIterator<Item = &'a u8>,
        V: Into<String>,
    {
        // FIXME? Without this internal function `self` is moved into `fold()`
        // and thus cannot be returned.
        #[inline]
        fn internal<'a, K: Iterator<Item = &'a u8>>(
            node: &mut Node,
            key: K,
            value: String,
        ) {
            let mut node = key.fold(node, |node, &c| {
                node.branch.entry(c).or_insert_with(Node::default)
            });
            node.leaf = Some(value);
        }
        internal(self, key.into_iter(), value.into());
        self
    }

    /// Render the matcher into Rust code that works on an iterator.
    ///
    /// The parameters are:
    ///
    ///   1. An instance of [`std::io::Write`], like [`std::io::stdout()`], a
    ///      file, or a [`Vec`].
    ///   2. The first part of the function definition to generate, e.g.
    ///      `"pub fn matcher"`.
    ///   3. The return type (will be wrapped in [`Option`]), e.g.
    ///      `"&'static str"`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use bstr::ByteVec;
    /// use iter_matcher::Node;
    /// use pretty_assertions::assert_str_eq;
    ///
    /// let mut out = Vec::new();
    /// Node::from_iter([("a".as_bytes(), "1")])
    ///     .render_iter(&mut out, "fn match_bytes", "u64")
    ///     .unwrap();
    ///
    /// assert_str_eq!(
    ///     "\
    /// fn match_bytes<'a, I>(iter: &mut I) -> Option<u64>
    /// where
    ///     I: core::iter::Iterator<Item = &'a u8> + core::clone::Clone,
    /// {
    ///     let fallback_iter = iter.clone();
    ///     match iter.next() {
    ///         Some(97) => Some(1),
    ///         _ => {
    ///             *iter = fallback_iter;
    ///             None
    ///         }
    ///     }
    /// }
    /// ",
    ///     out.into_string().unwrap(),
    /// );
    /// ```
    pub fn render_iter<W: io::Write, N: AsRef<str>, R: AsRef<str>>(
        &self,
        writer: &mut W,
        fn_name: N,
        return_type: R,
    ) -> io::Result<()> {
        let indent = "    "; // Our formatting prevents embedding this.
        let fn_name = fn_name.as_ref();
        let return_type = return_type.as_ref();

        if self.branch.is_empty() {
            // Special handling for when no matches were added.
            write!(
                writer,
                "{fn_name}<'a, I>(_iter: &mut I) -> Option<{return_type}>\n\
                where\n\
                {indent}I: core::iter::Iterator<Item = &'a u8> + core::clone::Clone,\n\
                {{\n\
                {indent}")?;
            render_child(self, writer, 0, None)?;
            writeln!(writer, "\n}}")?;
        } else {
            write!(
                writer,
                "{fn_name}<'a, I>(iter: &mut I) -> Option<{return_type}>\n\
                where\n\
                {indent}I: core::iter::Iterator<Item = &'a u8> + core::clone::Clone,\n"
            )?;
            render_child(self, writer, 0, None)?;
            writeln!(writer)?;
        }

        // FIXME: this is recursive, so for long patterns it could blow out the
        // stack. Transform this to an iterative algorithm.

        #[inline]
        pub fn render_child<W: io::Write>(
            node: &Node,
            writer: &mut W,
            level: usize,
            fallback: Option<&String>,
        ) -> io::Result<()> {
            if node.branch.is_empty() {
                // Terminal. node.leaf should be Some(_), but might not be.
                write!(
                    writer,
                    "{}",
                    leaf_to_str(node.leaf.as_ref().or(fallback))
                )?;
            } else if node.leaf.is_none() && level > 0 {
                // No patterns end here: branch only. (The level check creates
                // a default root pattern of `[] => None` so that we rewind the
                // iter when nothing matches.)
                render_match(node, writer, level, fallback)?;
            } else {
                // A pattern ends here.
                let indent = "    ".repeat(level);
                write!(
                    writer,
                    "{{\n\
                    {indent}    let fallback_iter = iter.clone();\n\
                    {indent}    "
                )?;
                render_match(node, writer, level + 1, node.leaf.as_ref())?;
                write!(writer, "\n{indent}}}")?;
            }

            Ok(())
        }

        #[inline]
        fn render_match<W: io::Write>(
            node: &Node,
            writer: &mut W,
            level: usize,
            fallback: Option<&String>,
        ) -> io::Result<()> {
            let indent = "    ".repeat(level);
            writeln!(writer, "match iter.next() {{")?;
            for (chunk, child) in &node.branch {
                write!(writer, "{indent}    Some({chunk:?}) => ")?;
                render_child(child, writer, level + 1, fallback)?;
                if child.branch.is_empty() {
                    // render_child() wrote a value, not a match block.
                    writeln!(writer, ",")?;
                } else {
                    // render_child() wrote a match block.
                    writeln!(writer)?;
                }
            }

            let fallback = leaf_to_str(fallback);
            write!(
                writer,
                "{indent}    _ => {{\n\
                {indent}        *iter = fallback_iter;\n\
                {indent}        {fallback}\n\
                {indent}    }}\n\
                {indent}}}"
            )?;

            Ok(())
        }

        #[inline]
        fn leaf_to_str(leaf: Option<&String>) -> String {
            if let Some(leaf) = leaf {
                format!("Some({leaf})")
            } else {
                "None".to_string()
            }
        }

        Ok(())
    }

    /// Render the matcher into Rust code that works on a slice.
    ///
    /// The parameters are:
    ///
    ///   1. An instance of [`std::io::Write`], like [`std::io::stdout()`], a
    ///      file, or a [`Vec`].
    ///   2. The first part of the function definition to generate, e.g.
    ///      `"pub fn matcher"`.
    ///   3. The return type (will be wrapped in [`Option`] and a tuple), e.g.
    ///      `"&'static str"`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use bstr::ByteVec;
    /// use iter_matcher::Node;
    /// use pretty_assertions::assert_str_eq;
    ///
    /// let mut out = Vec::new();
    /// Node::from_iter([("a".as_bytes(), "1")])
    ///     .render_slice(&mut out, "fn match_bytes", "u64")
    ///     .unwrap();
    ///
    /// assert_str_eq!(
    ///     "\
    /// fn match_bytes(slice: &[u8]) -> (Option<u64>, &[u8]) {
    ///     match slice.first() {
    ///         Some(97) => (Some(1), &slice[1..]),
    ///         _ => (None, slice),
    ///     }
    /// }
    /// ",
    ///     out.into_string().unwrap(),
    /// );
    /// ```
    pub fn render_slice<W: io::Write, N: AsRef<str>, R: AsRef<str>>(
        &self,
        writer: &mut W,
        fn_name: N,
        return_type: R,
    ) -> io::Result<()> {
        let indent = "    "; // Our formatting prevents embedding this.
        let fn_name = fn_name.as_ref();
        let return_type = return_type.as_ref();

        write!(
            writer,
            "{fn_name}(slice: &[u8]) -> (Option<{return_type}>, &[u8]) {{\n\
            {indent}",
        )?;
        render_child(self, writer, 0, None)?;
        writeln!(writer, "\n}}")?;

        // FIXME: this is recursive, so for long patterns it could blow out the
        // stack. Transform this to an iterative algorithm.

        #[inline]
        pub fn render_child<W: io::Write>(
            node: &Node,
            writer: &mut W,
            index: usize,
            fallback: Option<(&String, usize)>,
        ) -> io::Result<()> {
            if node.branch.is_empty() {
                // Terminal. node.leaf should be Some(_), but might not be.
                if let Some(leaf) = &node.leaf {
                    write!(writer, "(Some({leaf}), &slice[{index}..])")?;
                } else if let Some((value, fallback)) = fallback {
                    write!(writer, "(Some({value}), &slice[{fallback}..])")?;
                } else {
                    write!(writer, "(None, slice)")?;
                }
            } else if node.leaf.is_none() {
                // No patterns end here: branch only.
                render_match(node, writer, index, fallback)?;
            } else {
                // A pattern ends here.
                render_match(
                    node,
                    writer,
                    index,
                    node.leaf.as_ref().map(|leaf| (leaf, index)),
                )?;
            }

            Ok(())
        }

        #[inline]
        fn render_match<W: io::Write>(
            node: &Node,
            writer: &mut W,
            index: usize,
            fallback: Option<(&String, usize)>,
        ) -> io::Result<()> {
            let indent = "    ".repeat(index + 1);
            if index == 0 {
                // For Clippy.
                writeln!(writer, "match slice.first() {{")?;
            } else {
                writeln!(writer, "match slice.get({index}) {{")?;
            }

            for (chunk, child) in &node.branch {
                write!(writer, "{indent}    Some({chunk:?}) => ")?;
                render_child(child, writer, index + 1, fallback)?;
                if child.branch.is_empty() {
                    // render_child() wrote a value, not a match block.
                    writeln!(writer, ",")?;
                } else {
                    // render_child() wrote a match block.
                    writeln!(writer)?;
                }
            }

            let default = if let Some((value, fallback)) = fallback {
                format!("(Some({value}), &slice[{fallback}..])")
            } else {
                "(None, slice)".to_string()
            };

            write!(
                writer,
                "{indent}    _ => {default},\n\
                {indent}}}"
            )?;

            Ok(())
        }

        Ok(())
    }
}

impl<'a, K, V> FromIterator<(K, V)> for Node
where
    K: IntoIterator<Item = &'a u8>,
    V: Into<String>,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Node {
        let mut root = Node::default();
        root.extend(iter);
        root
    }
}

impl<'a, K, V> Extend<(K, V)> for Node
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

/// Render a stub function with the correct signature for consuming an iterator
/// that does nothing.
pub fn render_iter_stub<W: io::Write, N: AsRef<str>, R: AsRef<str>>(
    writer: &mut W,
    fn_name: N,
    return_type: R,
) -> io::Result<()> {
    Node::default().render_iter(writer, fn_name, return_type)
}

/// Render a stub function with the correct signature for consuming a slice that
/// does nothing.
pub fn render_slice_stub<W: io::Write, N: AsRef<str>, R: AsRef<str>>(
    writer: &mut W,
    fn_name: N,
    return_type: R,
) -> io::Result<()> {
    Node::default().render_slice(writer, fn_name, return_type)
}

/// A matcher function builder.
///
/// The generated function will accept an iterator over bytes and will return a
/// match if it finds a given byte sequence at the start of the iterator.
///
/// For example, suppose you generate a [matcher for all HTML entities][htmlize]
/// called `entity_matcher()`:
///
/// ```rust,ignore
/// let mut iter = b"&times;XYZ".iter();
/// assert!(entity_matcher(&mut iter) == Some("×"));
/// assert!(iter.next() == Some(b'X'));
/// ```
///
///   * **The prefixes it checks do not all have to be the same length.** The
///     matcher will clone the iterator if it needs to look ahead, so when the
///     matcher returns the iterator will only have consumed what was matched.
///   * **If more than one prefix matches, it will return the longest one.**
///   * **If nothing matches, the iterator will not be advanced.** You may want
///     to call `iterator.next()` if the matcher returns `None`.
///   * **It only checks the start of the iterator.** Often you will want to use
///     [`position()`] or the [memchr crate][memchr] to find the start of a
///     potential match.
///
/// See the [crate] documentation for an example of how to use this from a build
/// script.
///
/// [`position()`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.position
/// [memchr]: http://docs.rs/memchr
/// [htmlize]: https://crates.io/crates/htmlize
#[derive(Debug)]
pub struct IterMatcher {
    /// The first part of the function definition to generate, e.g.
    /// `"pub fn matcher"`.
    pub fn_name: String,

    /// The return type (will be wrapped in [`Option`]), e.g. `"&'static str"`.
    pub return_type: String,

    /// Whether or not to prevent Clippy from evaluating the generated code.
    ///
    /// See [`Self::disable_clippy()`].
    pub disable_clippy: bool,

    /// The root of the matcher node tree.
    pub root: Node,
}

impl IterMatcher {
    /// Create a new matcher (for use in a build script).
    ///
    /// This will generate a matcher with the the specified function name and
    /// return type. You can add matches to it with [`Self::add()`] and/or
    /// [`Self::extend()`], then turn it into code with
    /// [`Self::render_slice()`].
    ///
    /// See the [crate documentation][crate] for a complete example.
    pub fn new<N: AsRef<str>, R: AsRef<str>>(
        fn_name: N,
        return_type: R,
    ) -> Self {
        Self {
            fn_name: fn_name.as_ref().to_string(),
            return_type: return_type.as_ref().to_string(),
            disable_clippy: false,
            root: Node::default(),
        }
    }

    /// Add a match.
    ///
    /// ```rust
    /// let mut matcher = iter_matcher::IterMatcher::new("fn matcher", "u64");
    /// matcher.add(b"a", "1");
    /// ```
    pub fn add<'a, K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: IntoIterator<Item = &'a u8>,
        V: Into<String>,
    {
        self.root.add(key, value);
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
    /// use iter_matcher::IterMatcher;
    /// use pretty_assertions::assert_str_eq;
    ///
    /// let mut out = Vec::new();
    /// let mut matcher = IterMatcher::new("fn match_bytes", "u64");
    /// matcher.disable_clippy(true);
    /// matcher.extend([("a".as_bytes(), "1")]);
    /// matcher.render_iter(&mut out).unwrap();
    ///
    /// assert_str_eq!(
    ///     r#"#[cfg(not(feature = "cargo-clippy"))]
    /// fn match_bytes<'a, I>(iter: &mut I) -> Option<u64>
    /// where
    ///     I: core::iter::Iterator<Item = &'a u8> + core::clone::Clone,
    /// {
    ///     let fallback_iter = iter.clone();
    ///     match iter.next() {
    ///         Some(97) => Some(1),
    ///         _ => {
    ///             *iter = fallback_iter;
    ///             None
    ///         }
    ///     }
    /// }
    ///
    /// #[cfg(feature = "cargo-clippy")]
    /// fn match_bytes<'a, I>(_iter: &mut I) -> Option<u64>
    /// where
    ///     I: core::iter::Iterator<Item = &'a u8> + core::clone::Clone,
    /// {
    ///     None
    /// }
    /// "#,
    ///     out.into_string().unwrap(),
    /// );
    /// ```
    pub fn render_iter<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.disable_clippy {
            writeln!(writer, "#[cfg(not(feature = \"cargo-clippy\"))]")?;
        }

        self.root
            .render_iter(writer, &self.fn_name, &self.return_type)?;

        if self.disable_clippy {
            writeln!(writer)?;
            writeln!(writer, "#[cfg(feature = \"cargo-clippy\")]")?;
            render_iter_stub(writer, &self.fn_name, &self.return_type)?;
        }

        Ok(())
    }

    /// Render the matcher into Rust code.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use bstr::ByteVec;
    /// use iter_matcher::IterMatcher;
    /// use pretty_assertions::assert_str_eq;
    ///
    /// let mut out = Vec::new();
    /// let mut matcher = IterMatcher::new("fn match_bytes", "u64");
    /// matcher.disable_clippy(true);
    /// matcher.extend([("a".as_bytes(), "1")]);
    /// matcher.render_slice(&mut out).unwrap();
    ///
    /// assert_str_eq!(
    ///     r#"#[cfg(not(feature = "cargo-clippy"))]
    /// fn match_bytes(slice: &[u8]) -> (Option<u64>, &[u8]) {
    ///     match slice.first() {
    ///         Some(97) => (Some(1), &slice[1..]),
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
    pub fn render_slice<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.disable_clippy {
            writeln!(writer, "#[cfg(not(feature = \"cargo-clippy\"))]")?;
        }

        self.root
            .render_slice(writer, &self.fn_name, &self.return_type)?;

        if self.disable_clippy {
            writeln!(writer)?;
            writeln!(writer, "#[cfg(feature = \"cargo-clippy\")]")?;
            render_slice_stub(writer, &self.fn_name, &self.return_type)?;
        }

        Ok(())
    }
}

impl<'a, K, V> Extend<(K, V)> for IterMatcher
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

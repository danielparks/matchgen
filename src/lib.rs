//! # Example build script
//!
//! ```rust
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
//!     iter_matcher::Node::default()
//!         .add(b"one", r#"b"1""#)
//!         .add(b"two", r#"b"2""#)
//!         .add(b"three", r#"b"3""#)
//!         .render(&mut out, "pub fn fancy_matcher", "&'static [u8]")?;
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

use std::collections::HashMap;
use std::io;

/// A node in the simple finite-state automaton that is a matcher.
///
/// See [`Node::from_iter()`] and [`Node::render()`].
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

    /// Render the matcher into Rust code.
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
    ///     .render(&mut out, "fn match_bytes", "u64")
    ///     .unwrap();
    ///
    /// assert_str_eq!(
    ///     out.into_string().unwrap(),
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
    /// ");
    /// ```
    pub fn render<W: io::Write, N: AsRef<str>, R: AsRef<str>>(
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
                // FIXME: make this configurable.
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

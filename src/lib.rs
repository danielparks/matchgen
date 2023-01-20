use std::collections::HashMap;
use std::io;

/// A node in the simple finite-state automaton that is a matcher.
///
/// See [`generate()`] and [`Node::render()`].
#[derive(Debug, Default)]
pub struct Node {
    pub leaf: Option<String>,
    pub branch: HashMap<u8, Node>,
}

impl Node {
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
    /// use iter_matcher::generate;
    /// let mut out = Vec::new();
    ///
    /// generate([("a".bytes(), "1".to_string())])
    ///     .render(&mut out, "fn match", "u64")
    ///     .unwrap();
    ///
    /// assert_eq!(
    ///     String::from_utf8(out).unwrap(),
    ///     "\
    /// #[allow(unreachable_patterns)]
    /// fn match<I>(iter: &mut I) -> Option<u64>
    /// where
    ///     I: core::iter::Iterator<Item = u8> + core::clone::Clone,
    /// {
    ///     let fallback_iter = iter.clone();
    ///     match iter.next() {
    ///         97 => Some(1),
    ///         _ => {
    ///             *iter = fallback_iter;
    ///             None
    ///         }
    ///     }
    /// }");
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
        write!(
            writer,
            "#[allow(unreachable_patterns)]\n\
            {fn_name}<I>(iter: &mut I) -> Option<{return_type}>\n\
            where\n\
            {indent}I: core::iter::Iterator<Item = u8> + core::clone::Clone,\n"
        )?;
        render_child(self, writer, 0, None)?;

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
                // No patterns end here: branch only. (There is an implicit
                // default root pattern of [] => None so that we rewind the iter
                // when it matches.)
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
                write!(writer, "{indent}    {chunk:?} => ")?;
                render_child(child, writer, level + 1, fallback)?;
                if child.branch.is_empty() {
                    // render_child() outputs a value, not a match block.
                    writeln!(writer, ",")?;
                } else {
                    // ender_child() outputs a match block.
                    writeln!(writer)?;
                }
            }

            // FIXME? we could leave this off if all possible branches are used,
            // which would allow us to reenable #[warn(unreachable_patterns)].
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
                format!("Some({})", leaf)
            } else {
                "None".to_string()
            }
        }

        Ok(())
    }
}

/// Generate a matcher.
///
/// This takes an iterator of tuple pairs `(K, V)`. `K` must be an iterator
/// that generates bytes, e.g. `"key".bytes()`, and `V` must be a [`String`].
///
/// Call [`Node::render()`] on the result to actually generate the code.
///
/// ```rust
/// use iter_matcher::generate;
/// let node = generate([("a".bytes(), "1".to_string())]);
/// ```

pub fn generate<K, I>(key_values: I) -> Node
where
    K: IntoIterator<Item = u8>,
    I: IntoIterator<Item = (K, String)>,
{
    let mut root = Node::default();
    key_values.into_iter().for_each(|(key, value)| {
        let mut node = key.into_iter().fold(&mut root, |node, c| {
            node.branch.entry(c).or_insert_with(Node::default)
        });

        node.leaf = Some(value);
    });
    root
}

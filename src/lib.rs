use std::collections::HashMap;
use std::io;

#[derive(Debug, Default)]
pub struct Node {
    pub leaf: Option<String>,
    pub branch: HashMap<u8, Node>,
}

impl Node {
    pub fn render<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let indent = "    ";
        write!(
            writer,
            "#[allow(unreachable_patterns)]\n\
            fn match<I>(iter: &mut I) -> Option<&'static str>\n\
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

/// Generate matcher
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

#[cfg(test)]
mod tests {
    macro_rules! test {
        ($name:ident, $($test:tt)+) => {
            #[test]
            fn $name() {
                ::assert2::assert!($($test)+);
            }
        };
    }

    test!(byte_tilde, b'~' == b'~');
}

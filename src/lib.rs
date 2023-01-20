use std::collections::HashMap;
use std::fmt;
use std::io;

pub struct Node<V> {
    pub leaf: Option<V>,
    pub branch: HashMap<u8, Node<V>>,
}

impl<V> Node<V>
where
    V: fmt::Debug,
{
    pub fn render<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "#[allow(unreachable_patterns)]")?;
        writeln!(writer, "fn match<I>(iter: &mut I) -> Option<&'static str>")?;
        writeln!(writer, "where")?;
        writeln!(
            writer,
            "    I: core::iter::Iterator<Item = u8> + core::clone::Clone,"
        )?;
        render_child(self, writer, 0, None)?;

        // FIXME: this is recursive, so for long patterns it could blow out the
        // stack. Transform this to an iterative algorithm.

        #[inline]
        pub fn render_child<V: fmt::Debug, W: io::Write>(
            node: &Node<V>,
            writer: &mut W,
            level: usize,
            fallback: Option<&V>,
        ) -> io::Result<()> {
            if node.branch.is_empty() {
                // Terminal. node.leaf should be Some(_), but might not be.
                write!(writer, "{:?}", node.leaf.as_ref().or(fallback))?;
            } else if node.leaf.is_none() && level > 0 {
                // No patterns end here: branch only. (There is an implicit
                // default root pattern of [] => None so that we rewind the iter
                // when it matches.)
                render_match(node, writer, level, fallback)?;
            } else {
                // A pattern ends here.
                let indent = "    ".repeat(level);
                writeln!(writer, "{{")?;
                writeln!(
                    writer,
                    "{indent}    let fallback_iter = iter.clone();"
                )?;
                write!(writer, "{indent}    ")?;
                render_match(node, writer, level + 1, node.leaf.as_ref())?;
                writeln!(writer)?;
                write!(writer, "{indent}}}")?;
            }

            Ok(())
        }

        #[inline]
        fn render_match<V: fmt::Debug, W: io::Write>(
            node: &Node<V>,
            writer: &mut W,
            level: usize,
            fallback: Option<&V>,
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
            writeln!(writer, "{indent}    _ => {{")?;
            writeln!(writer, "{indent}        *iter = fallback_iter;")?;
            writeln!(writer, "{indent}        {:?}", fallback)?;
            writeln!(writer, "{indent}    }}")?;
            write!(writer, "{indent}}}")?;

            Ok(())
        }

        Ok(())
    }
}

impl<V> Default for Node<V> {
    fn default() -> Self {
        Self {
            leaf: None,
            branch: HashMap::new(),
        }
    }
}

impl<V: fmt::Debug> fmt::Debug for Node<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("leaf", &self.leaf)
            .field("branch", &self.branch)
            .finish()
    }
}

/// Generate matcher
pub fn generate<K, V, I>(key_values: I) -> Node<V>
where
    K: IntoIterator<Item = u8>,
    I: IntoIterator<Item = (K, V)>,
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

use std::collections::HashMap;
use std::fmt;

pub struct Node<V> {
    pub leaf: Option<V>,
    pub branch: HashMap<u8, Node<V>>,
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
pub fn generate<'a, K, V, I>(key_values: I)
where
    K: IntoIterator<Item = &'a u8>,
    I: IntoIterator<Item = (K, V)>,
    V: fmt::Debug,
{
    let mut root = Node::default();
    key_values.into_iter().for_each(|(key, value)| {
        let mut node = key.into_iter().fold(&mut root, |node, &c| {
            node.branch.entry(c).or_insert_with(Node::default)
        });

        node.leaf = Some(value);
    });

    render(&root, 0, None);
}

pub fn render<V>(node: &Node<V>, level: usize, fallback: Option<&V>)
where
    V: fmt::Debug,
{
    if node.branch.is_empty() {
        // Terminal. Could be None or Some(_).
        print!("{:?}", node.leaf);
    } else {
        let indent = "    ".repeat(level);
        println!("match iter.next() {{");
        node.branch.iter().for_each(|(chunk, child)| {
            print!("{indent}    {chunk:?} => ");
            render(child, level + 1, node.leaf.as_ref().or(fallback));
            println!(",");
        });
        // FIXME: what happens if all possibilities are exhausted?
        println!("{indent}    _ => {:?},", node.leaf.as_ref().or(fallback));
        print!("{indent}}}");
    }
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

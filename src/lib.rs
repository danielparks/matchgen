use std::collections::HashMap;
use std::fmt;

pub struct Node<V> {
    pub leaf: Option<V>,
    pub branch: HashMap<u8, Node<V>>,
}

impl<V> Node<V>
where
    V: fmt::Debug,
{
    #[inline]
    pub fn render(&self) {
        self.render_child(0, None)
    }
    pub fn render_child(&self, level: usize, fallback: Option<&V>) {
        if self.branch.is_empty() {
            // Terminal. Could be None or Some(_).
            print!("{:?}", self.leaf);
        } else {
            let indent = "    ".repeat(level);
            println!("match iter.next() {{");
            self.branch.iter().for_each(|(chunk, child)| {
                print!("{indent}    {chunk:?} => ");
                child.render_child(level + 1, self.leaf.as_ref().or(fallback));
                println!(",");
            });
            // FIXME: what happens if all possibilities are exhausted?
            println!("{indent}    _ => {:?},", self.leaf.as_ref().or(fallback));
            print!("{indent}}}");
        }
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
pub fn generate<'a, K, V, I>(key_values: I) -> Node<V>
where
    K: IntoIterator<Item = &'a u8>,
    I: IntoIterator<Item = (K, V)>,
{
    let mut root = Node::default();
    key_values.into_iter().for_each(|(key, value)| {
        let mut node = key.into_iter().fold(&mut root, |node, &c| {
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

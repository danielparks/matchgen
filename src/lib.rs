use std::collections::HashMap;
use std::fmt;

struct Node<V> {
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

    println!("{root:?}");
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

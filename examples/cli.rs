use iter_matcher::Node;
use std::env;
use std::io;
use std::process::exit;

fn main() {
    let mut node = Node::default();
    env::args().skip(1).for_each(|arg| {
        let key_value: Vec<&str> = arg.splitn(2, '=').collect();
        if key_value.len() < 2 {
            eprintln!("Arguments must be key=value pairs: {arg:?}");
            exit(1);
        }
        node.add(key_value[0].as_bytes(), format!("{:?}", key_value[1]));
    });
    node.render(&mut io::stdout(), "pub fn match", "&'static str")
        .unwrap();
}

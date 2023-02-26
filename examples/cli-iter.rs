use iter_matcher::TreeMatcher;
use std::env;
use std::io;
use std::process::exit;

fn main() {
    let mut matcher = TreeMatcher::new("pub fn matcher", "&'static str");
    env::args().skip(1).for_each(|arg| {
        let key_value: Vec<&str> = arg.splitn(2, '=').collect();
        if key_value.len() < 2 {
            eprintln!("Arguments must be key=value pairs: {arg:?}");
            exit(1);
        }
        matcher.add(key_value[0].as_bytes(), format!("{:?}", key_value[1]));
    });
    matcher.render_iter(&mut io::stdout()).unwrap();
}

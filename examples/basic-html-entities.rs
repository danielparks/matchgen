use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let out_path = Path::new(&env::var("OUT_DIR")?).join("matcher.rs");
    let mut out = BufWriter::new(File::create(out_path)?);

    writeln!(out, "/// Decode basic HTML entities.")?;
    let mut node = iter_matcher::Node::default();
    node.add(b"&amp;", "b'&'");
    node.add(b"&lt;", "b'<'");
    node.add(b"&gt;", "b'>'");
    node.add(b"&quot;", "b'\"'");
    node.render(&mut out, "pub fn entity_decode", "u8")?;

    Ok(())
}

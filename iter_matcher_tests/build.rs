use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let out_path = Path::new(&env::var("OUT_DIR")?).join("test-matchers.rs");
    let mut out = BufWriter::new(File::create(out_path)?);

    writeln!(out, "/// Decode basic HTML entities.")?;
    iter_matcher::Node::default()
        .add(b"&amp;", "b'&'")
        .add(b"&lt;", "b'<'")
        .add(b"&gt;", "b'>'")
        .add(b"&quot;", "b'\"'")
        .render(&mut out, "pub fn basic_entity_decode", "u8")?;

    Ok(())
}

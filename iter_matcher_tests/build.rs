use std::env;
use std::error::Error;
use std::fs::{self, File};
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
    writeln!(out)?;

    writeln!(out, "/// Decode all HTML entities.")?;
    let input = fs::read("html-entities.json")?;
    let input: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(&input)?;
    iter_matcher::Node::from_iter(input.iter().map(|(name, info)| {
        (
            name.as_bytes(),
            format!("{:?}", info["characters"].as_str().unwrap()),
        )
    }))
    .render(&mut out, "pub fn all_entity_decode", "&'static str")?;
    writeln!(out)?;

    Ok(())
}
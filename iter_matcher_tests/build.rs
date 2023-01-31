use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let out_path = Path::new(&env::var("OUT_DIR")?).join("test-matchers.rs");
    let mut out = BufWriter::new(File::create(out_path)?);

    writeln!(out, "/// Match nothing.")?;
    iter_matcher::Node::default().render(
        &mut out,
        "pub fn match_nothing",
        "u8",
    )?;
    writeln!(out)?;

    writeln!(out, "/// Match nothing with Some(true).")?;
    iter_matcher::Node::default().add(b"", "true").render(
        &mut out,
        "pub fn match_nothing_true",
        "bool",
    )?;
    writeln!(out)?;

    writeln!(out, "/// Match and return (bool, &[u8]).")?;
    iter_matcher::Node::default()
        .add(b"aab", "(true, &[1, 1])")
        .add(b"aa", "(false, &[1, 1])")
        .add(b"ab", "(true, &[1])")
        .add(b"a", "(false, &[1])")
        .render(&mut out, "pub fn slice_in_tuple", "(bool, &'static [u8])")?;
    writeln!(out)?;

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

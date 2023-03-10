use matchgen::{Input, TreeMatcher};
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let out_path = Path::new(&env::var("OUT_DIR")?).join("test-matchers.rs");
    let mut out = BufWriter::new(File::create(out_path)?);

    writeln!(out, "/// Match nothing.")?;
    matchgen::TreeNode::default().render_iter(
        &mut out,
        "pub fn match_nothing",
        "u8",
    )?;
    writeln!(out)?;

    writeln!(out, "/// Match nothing (slice).")?;
    matchgen::TreeNode::default().render_slice(
        &mut out,
        "pub fn match_nothing_slice",
        "u8",
    )?;
    writeln!(out)?;

    writeln!(out, "/// Match nothing with Some(true).")?;
    matchgen::TreeNode::default().add(b"", "true").render_iter(
        &mut out,
        "pub fn match_nothing_true",
        "bool",
    )?;
    writeln!(out)?;

    writeln!(out, "/// Match nothing with Some(true).")?;
    matchgen::TreeNode::default()
        .add(b"", "true")
        .render_slice(&mut out, "pub fn match_nothing_slice_true", "bool")?;
    writeln!(out)?;

    writeln!(out, "/// Match and return (bool, &[u8]).")?;
    TreeMatcher::new("pub fn slice_in_tuple", "(bool, &'static [u8])")
        .add(b"aab", "(true, &[1, 1])")
        .add(b"aa", "(false, &[1, 1])")
        .add(b"ab", "(true, &[1])")
        .add(b"a", "(false, &[1])")
        .disable_clippy(true)
        .set_input_type(Input::Iterator)
        .render(&mut out)?;
    writeln!(out)?;

    writeln!(out, "/// Match and return (bool, &[u8]).")?;
    TreeMatcher::new("pub fn slice_in_tuple_slice", "(bool, &'static [u8])")
        .add(b"aab", "(true, &[1, 1])")
        .add(b"aa", "(false, &[1, 1])")
        .add(b"ab", "(true, &[1])")
        .add(b"a", "(false, &[1])")
        .disable_clippy(true)
        .set_input_type(Input::Slice)
        .render(&mut out)?;
    writeln!(out)?;

    writeln!(out, "/// Decode basic HTML entities.")?;
    TreeMatcher::new("pub fn basic_entity_decode", "u8")
        .add(b"&amp;", "b'&'")
        .add(b"&lt;", "b'<'")
        .add(b"&gt;", "b'>'")
        .add(b"&quot;", "b'\"'")
        .disable_clippy(false)
        .set_input_type(Input::Iterator)
        .render(&mut out)?;
    writeln!(out)?;

    writeln!(out, "/// Decode basic HTML entities.")?;
    TreeMatcher::new("pub fn basic_entity_decode_slice", "u8")
        .add(b"&amp;", "b'&'")
        .add(b"&lt;", "b'<'")
        .add(b"&gt;", "b'>'")
        .add(b"&quot;", "b'\"'")
        .disable_clippy(false)
        .set_input_type(Input::Slice)
        .render(&mut out)?;
    writeln!(out)?;

    writeln!(out, "/// Decode all HTML entities.")?;
    let input = fs::read("html-entities.json")?;
    let input: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(&input)?;
    let mut matcher =
        TreeMatcher::new("pub fn all_entity_decode", "&'static str");
    matcher
        .disable_clippy(true)
        .set_input_type(Input::Iterator)
        .extend(input.iter().map(|(name, info)| {
            (
                name.as_bytes(),
                format!("{:?}", info["characters"].as_str().unwrap()),
            )
        }));
    matcher.render(&mut out)?;
    writeln!(out)?;

    writeln!(out, "/// Decode all HTML entities.")?;
    matcher.fn_name = "pub fn all_entity_decode_slice".to_string();
    matcher.set_input_type(Input::Slice);
    matcher.render(&mut out)?;
    writeln!(out)?;

    Ok(())
}

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::let_underscore_untyped,
    clippy::map_unwrap_or,
    clippy::too_many_lines
)]

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
    writeln!(out, "#[must_use]")?;
    matchgen::TreeNode::default().render_iter(
        &mut out,
        "pub fn match_nothing",
        "u8",
    )?;
    writeln!(out)?;

    writeln!(out, "/// Match nothing (slice).")?;
    writeln!(out, "#[must_use]")?;
    matchgen::TreeNode::default().render_slice(
        &mut out,
        "pub fn match_nothing_slice",
        "u8",
    )?;
    writeln!(out)?;

    writeln!(out, "/// Match nothing with Some(true).")?;
    writeln!(out, "#[must_use]")?;
    matchgen::TreeNode::default().add(b"", "true").render_iter(
        &mut out,
        "pub fn match_nothing_true",
        "bool",
    )?;
    writeln!(out)?;

    writeln!(out, "/// Match nothing with Some(true).")?;
    writeln!(out, "#[must_use]")?;
    matchgen::TreeNode::default()
        .add(b"", "true")
        .render_slice(&mut out, "pub fn match_nothing_slice_true", "bool")?;
    writeln!(out)?;

    TreeMatcher::new("pub fn slice_in_tuple", "(bool, &'static [u8])")
        .add(b"aab", "(true, &[1, 1])")
        .add(b"aa", "(false, &[1, 1])")
        .add(b"ab", "(true, &[1])")
        .add(b"a", "(false, &[1])")
        // FIXME? I don’t understand why the blank lines cause failures here.
        .doc(Some(
            "Match and return `(bool, &[u8])`.
            \n\
            Iterator version.
            \n\
            Really long doc string to ensure that clippy doesn’t complain even
            though this will have to be will beyond whatever the line length
            limit is.",
        ))
        .disable_clippy(true)
        .input_type(Input::Iterator)
        .render(&mut out)?;
    writeln!(out)?;

    TreeMatcher::new("pub fn slice_in_tuple_slice", "(bool, &'static [u8])")
        .add(b"aab", "(true, &[1, 1])")
        .add(b"aa", "(false, &[1, 1])")
        .add(b"ab", "(true, &[1])")
        .add(b"a", "(false, &[1])")
        .doc(Some("Match and return `(bool, &[u8])`.\n\nSlice version."))
        .disable_clippy(true)
        .input_type(Input::Slice)
        .render(&mut out)?;
    writeln!(out)?;

    TreeMatcher::new("pub fn basic_entity_decode", "u8")
        .add(b"&amp;", "b'&'")
        .add(b"&lt;", "b'<'")
        .add(b"&gt;", "b'>'")
        .add(b"&quot;", "b'\"'")
        .doc(Some("Decode basic HTML entities.\n\nIterator version."))
        .disable_clippy(false)
        .input_type(Input::Iterator)
        .render(&mut out)?;
    writeln!(out)?;

    TreeMatcher::new("pub fn basic_entity_decode_slice", "u8")
        .add(b"&amp;", "b'&'")
        .add(b"&lt;", "b'<'")
        .add(b"&gt;", "b'>'")
        .add(b"&quot;", "b'\"'")
        .doc(Some("Decode basic HTML entities.\n\nSlice version."))
        .disable_clippy(false)
        .input_type(Input::Slice)
        .render(&mut out)?;
    writeln!(out)?;

    let input = fs::read("most-html-entities.json")?;
    let input: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(&input)?;
    let mut matcher =
        TreeMatcher::new("pub fn most_entity_decode", "&'static str");
    matcher
        .doc(Some("Decode most HTML entities.\n\nIterator version."))
        .disable_clippy(true)
        .input_type(Input::Iterator)
        .extend(input.iter().map(|(name, info)| {
            (
                name.as_bytes(),
                format!("{:?}", info["characters"].as_str().unwrap()),
            )
        }));
    matcher.render(&mut out)?;
    writeln!(out)?;

    matcher.fn_name = "pub fn most_entity_decode_slice".to_string();
    matcher
        .doc(Some("Decode most HTML entities.\n\nSlice version."))
        .input_type(Input::Slice)
        .render(&mut out)?;
    writeln!(out)?;

    Ok(())
}

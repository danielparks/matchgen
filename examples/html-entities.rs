use clap::Parser;
use iter_matcher::Node;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::exit;

#[derive(Debug, clap::Parser)]
#[clap(version, about)]
struct Params {
    entities: PathBuf,
}

fn main() {
    if let Err(error) = cli(Params::parse()) {
        eprintln!("Error: {:#}", error);
        exit(1);
    }
}

fn cli(params: Params) -> anyhow::Result<()> {
    let input = fs::read(params.entities).unwrap();
    let input: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(&input).unwrap();

    let node = Node::from_iter(input.iter().map(|(name, info)| {
        (
            name.as_bytes(),
            format!("{:?}", info["characters"].as_str().unwrap()),
        )
    }));

    node.render(&mut io::stdout(), "fn match", "&'static str")?;

    Ok(())
}

use clap::Parser;
use iter_matcher::generate;
use std::fs;
use std::path::{Path, PathBuf};
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
    generate(load_entities(&params.entities)).render();
    Ok(())
}

fn load_entities(path: &Path) -> Vec<(Vec<u8>, String)> {
    let input = fs::read(path).unwrap();
    let input: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(&input).unwrap();

    input
        .iter()
        .map(|(name, info)| {
            (
                name.as_bytes().to_vec(),
                info["characters"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

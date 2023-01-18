use clap::Parser;
use std::process::exit;
use iter_matcher::generate;

#[derive(Debug, clap::Parser)]
#[clap(version, about)]
struct Params {
    /// key=value
    key_values: Vec<String>,
}

fn main() {
    if let Err(error) = cli(Params::parse()) {
        eprintln!("Error: {:#}", error);
        exit(1);
    }
}

fn cli(params: Params) -> anyhow::Result<()> {
    generate(params.key_values.iter().map(|key_value| {
        let key_value: Vec<&str> = key_value.splitn(2, '=').collect();
        let key = key_value[0].clone();
        let value = key_value[1].clone();
        (key.as_bytes(), value)
    }));
    Ok(())
}

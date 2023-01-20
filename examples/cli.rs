use clap::Parser;
use iter_matcher::generate;
use std::io;
use std::process::exit;

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

#[allow(clippy::clone_double_ref)]
fn cli(params: Params) -> anyhow::Result<()> {
    let node = generate(params.key_values.iter().map(|key_value| {
        let key_value: Vec<&str> = key_value.splitn(2, '=').collect();
        (key_value[0].bytes(), format!("{:?}", key_value[1]))
    }));

    node.render(&mut io::stdout(), "pub fn match", "&'static str")?;
    Ok(())
}

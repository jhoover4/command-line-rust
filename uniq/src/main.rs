use clap::Parser;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

/// Rust version of `uniq`
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file
    #[arg(default_value = "-", value_name = "FILE")]
    in_file: String,
    /// Output file
    out_file: Option<String>,
    /// Show counts
    #[arg(short, long)]
    count: bool,
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(_args: Args) -> Result<()> {
    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

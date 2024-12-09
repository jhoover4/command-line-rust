use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

/// Rust version of `head`
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file(s)
    #[arg(default_value = "-", value_name = "FILE")]
    files: Vec<String>,
    /// Number of lines
    #[arg(short('n'), long, default_value = "10", conflicts_with("bytes"), value_parser = clap::value_parser!(u64).range(1..))]
    lines: u64,
    /// Number of bytes
    #[arg(short('c'), long, value_parser = clap::value_parser!(u64).range(1..))]
    bytes: Option<u64>,
}

// TODO: Any value for -n or -c that cannot be parsed into a positive integer should cause the program to halt with an error
fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    for filename in args.files {
        match open(&filename) {
            Err(err) => eprintln!("{filename}: {err}"),
            Ok(_) => println!("Opened {filename}"),
        }
    }
    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

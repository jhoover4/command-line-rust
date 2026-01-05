use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

/// Rust version of `head`
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file(s)
    #[arg(default_value = "-", value_name = "FILE")]
    files: Vec<String>,
    /// Number of lines
    #[arg(short('n'), long, conflicts_with("bytes"), value_parser = clap::value_parser!(u64).range(1..))]
    lines: Option<u64>,
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
    let num_lines = args.lines;
    let num_bytes = args.bytes;

    let more_than_one_file = args.files.len() > 1;

    for (n, filename) in args.files.iter().enumerate() {
        match open(filename) {
            Err(err) => eprintln!("{filename}: {err}\n"),
            Ok(file) => {
                if more_than_one_file && n > 0 {
                    println!();
                }
                if more_than_one_file {
                    println!("==> {filename} <==");
                }

                if let Some(n) = num_bytes {
                    read_chars(file, n)?;
                } else if let Some(n) = num_lines {
                    read_lines(file, n)?;
                } else {
                    read_lines(file, 10)?;
                }
            }
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

fn read_lines(mut file: Box<dyn BufRead>, num_lines: u64) -> Result<()> {
    let mut line = String::new();

    for _ in 0..num_lines {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        print!("{line}");
        line.clear();
    }

    Ok(())
}

fn read_chars(mut file: Box<dyn BufRead>, num_bytes: u64) -> Result<()> {
    let mut buffer = vec![0; num_bytes as usize];
    let bytes_read = file.read(&mut buffer)?;
    print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));

    Ok(())
}

use std::io;
use std::fs::File;
use std::io::BufRead;
use clap::Parser;
use anyhow::Result;

/// Rust version of `cat`
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file(s)
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,
    /// Number lines
    #[arg(short('n'), long, conflicts_with("number_nonblank_lines"))]
    number_lines: bool,
    /// Number non-blank lines
    #[arg(short('b'), long("number-nonblank"))]
    number_nonblank_lines: bool,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(_args: Args) -> Result<()> {
    let mut read_files = Vec::new();

    for file in &_args.files {
        // TODO: Error from each file
        let lines = match file.as_str() {
            "-" => {
                read_from_stdin()
            }
            _ => {
                read_from_file(file)
            }
        }.expect("TODO: panic message");
        read_files.push(lines);
    }

    for file in &read_files {
        let mut n = 1;
        for line in file {
            if _args.number_lines {
                if line != "" {
                    println!("{:>6}\t{}", n, line);
                    n += 1;
                } else {
                    println!("{:>6}", line);
                }
            } else if _args.number_nonblank_lines {
                println!("{:>6}\t{}", n, line);
                n += 1;
            } else {
                println!("{}", line);
            }
        }
    }
    Ok(())
}

fn read_from_stdin() -> Result<Vec<String>> {
    let mut lines = Vec::new();

    let input = io::stdin().lines();
    for line in input {
        lines.push(line?);
    }
    Ok(lines)
}

fn read_from_file(file: &String) -> Result<Vec<String>> {
    let mut lines = Vec::new();

    let file = File::open(file)?;
    let reader = io::BufReader::new(file);

    let input = reader.lines();
    for line in input {
        lines.push(line?);
    }
    Ok(lines)
}

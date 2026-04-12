use anyhow::{anyhow, Result};
use clap::Parser;
use std::io::Write;
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

fn run(args: Args) -> Result<()> {
    let file = open(&args.in_file).map_err(|e| anyhow!("{}: {e}", args.in_file))?;

    let lines = BufReader::new(file).lines().map(|l| l.unwrap_or_default());
    let mut uniq_values_with_count: Vec<(i32, String)> = Vec::new();

    for curr_str in lines {
        match uniq_values_with_count.pop() {
            Some((prev_count, prev_str)) => {
                let curr_count = {
                    if curr_str == prev_str {
                        prev_count + 1
                    } else {
                        uniq_values_with_count.push((prev_count, prev_str));
                        1
                    }
                };
                uniq_values_with_count.push((curr_count, curr_str));
            }
            None => uniq_values_with_count.push((1, curr_str)),
        };
    }

    write(uniq_values_with_count, args.out_file, args.count)?;

    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn write(
    results: Vec<(i32, String)>,
    output_file: Option<String>,
    show_count: bool,
) -> Result<(), anyhow::Error> {
    let mut buf = String::new();
    let (counts, new_line): (Vec<i32>, Vec<String>) = results.into_iter().unzip();
    for (i, s) in new_line.into_iter().enumerate() {
        if show_count {
            buf.push_str(&format!("{:>4} {}\n", counts[i], s));
        } else {
            buf.push_str(&format!("{s}\n"));
        }
    }

    if buf.trim_end().is_empty() {
        return Ok(());
    }

    if let Some(filename) = output_file {
        let mut f = File::create(filename)?;
        f.write_all(buf.as_bytes())?;
    } else {
        print!("{buf}");
    }

    Ok(())
}

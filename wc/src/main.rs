use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::ops::{AddAssign};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file(s)
    #[arg(default_value = "-", value_name = "FILE")]
    files: Vec<String>,

    /// Show line count
    #[arg(short, long)]
    lines: bool,

    /// Show word count
    #[arg(short, long)]
    words: bool,

    /// Show byte count
    #[arg(short('c'), long)]
    bytes: bool,

    /// Show character count
    #[arg(short('m'), long, conflicts_with("bytes"))]
    chars: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

impl AddAssign for FileInfo {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            num_lines: self.num_lines + other.num_lines,
            num_words: self.num_words + other.num_words,
            num_bytes: self.num_bytes + other.num_bytes,
            num_chars: self.num_chars + other.num_chars,
        }
    }
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let more_than_one_file = args.files.len() > 1;
    let mut total = FileInfo {
        num_lines: 0,
        num_words: 0,
        num_bytes: 0,
        num_chars: 0,
    };

    for filename in &args.files {
        match open(filename) {
            Err(err) => eprintln!("{filename}: {err}"),
            Ok(file) => {
                let info = count(file)?;
                total += info.clone();

                print_count(info, &args);
                if filename != "-" {
                    print!(" {filename}");
                }
                println!();
            }
        }
    }

    if more_than_one_file {
        print_count(total, &args);
        print!(" total");
        println!();
    }

    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn count(mut file: impl BufRead) -> Result<FileInfo> {
    let mut buffer = String::new();
    let _ = &file.read_to_string(&mut buffer)?;

    let num_bytes = buffer.as_bytes().len();
    let num_lines = buffer.lines().count();
    let num_words = buffer.split_whitespace().count();
    let num_chars = buffer.chars().count();

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

fn print_count(info: FileInfo, args: &Args) {
    let mut results = vec![];

    let default = !args.lines && !args.words && !args.bytes && !args.chars;

    if args.lines || default {
        results.push(info.num_lines);
    }
    if args.words || default {
        results.push(info.num_words);
    }
    if args.bytes || default {
        results.push(info.num_bytes);
    }
    if args.chars {
        results.push(info.num_chars);
    }

    for result in results {
        print!("{:>8}", result);
    }
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world.\nI just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 2,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }
}

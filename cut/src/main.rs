#![allow(dead_code, unused)]
extern crate core;

use anyhow::{anyhow, Result};
use clap;
use clap::{Args, Parser};
use regex::Regex;
use std::ops::Range;
type PositionList = Vec<Range<usize>>;
#[derive(Parser, Debug)]
#[command(version, about)]
/// Rust version of `cut`
struct Cli {
    /// Input file(s)
    #[arg(default_value = "-")]
    files: Vec<String>,
    /// Field delimiter
    #[arg(short, long, default_value="\t", value_parser=is_byte)]
    delimiter: String,
    #[command(flatten)]
    input: Input,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct Input {
    #[arg(short, long, value_parser=parse_pos)]
    fields: PositionList,
    #[arg(short, long, value_parser=parse_pos)]
    bytes: PositionList,
    #[arg(short, long, value_parser=parse_pos)]
    chars: PositionList,
}

/// Checks if value is specifically one byte
fn is_byte(delim_str: &str) -> Result<char, String> {
    let delim_err = format!("--delim \"{delim_str}\" must be a single byte");

    let mut delim_chars = delim_str.chars();
    if delim_chars.clone().count() != 1 {
        return Err(delim_err);
    }
    let delim = delim_chars.next().ok_or(Err(delim_err))?;

    if delim.is_ascii() {
        Ok(delim)
    } else {
        Err(delim_err)
    }
}

fn parse_pos(range: &str) -> Result<PositionList, String> {
    let parse_pos_err = format!("illegal list value: {range}");

    let chars = range.chars();

    if chars.clone().count() > 3 {
        return Err(parse_pos_err);
    }

    let digits = chars.clone().map(|c| c.to_digit(10).ok_or(parse_pos_err)).collect::<Vec<_>>();

    let a = vec![Range::from(digits)];

    Ok(a)
}

fn main() {
    let args = Cli::parse();
    if let Err(e) = run(args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Cli) -> Result<()> {
    dbg!(args);
    Ok(())
}

#[cfg(test)]
mod unit_tests {
    use super::parse_pos;

    #[test]
    fn test_parse_pos() {
        // The empty string is an error
        assert!(parse_pos("").is_err());

        // Zero is an error
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "0""#);

        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "0""#);

        // A leading "+" is an error
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "+1""#,);

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "+1-2""#,
        );

        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "1-+2""#,
        );

        // Any non-number is an error
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);

        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);

        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "1-a""#,);

        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a-1""#,);

        // Wonky ranges
        let res = parse_pos("-");
        assert!(res.is_err());

        let res = parse_pos(",");
        assert!(res.is_err());

        let res = parse_pos("1,");
        assert!(res.is_err());

        let res = parse_pos("1-");
        assert!(res.is_err());

        let res = parse_pos("1-1-1");
        assert!(res.is_err());

        let res = parse_pos("1-1-a");
        assert!(res.is_err());

        // First number must be less than second
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }
}

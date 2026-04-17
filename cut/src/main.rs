#![allow(dead_code, unused)]
extern crate core;

use anyhow::{anyhow, Result};
use clap;
use clap::{Args, Parser};
use regex::Regex;
use std::ops::{Range, RangeFrom};
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
    let delim = delim_chars.next().ok_or(&delim_err)?;

    if delim.is_ascii() {
        Ok(delim)
    } else {
        Err(delim_err)
    }
}

fn parse_pos(range: &str) -> Result<PositionList, String> {
    let parse_pos_err = format!("illegal list value: \"{range}\"");

    let re = Regex::new(r"\d\-\d").map_err(|_| &parse_pos_err)?;
    if !re.is_match(range) {
        return Err(parse_pos_err);
    }

    let mut chars = range.chars();
    let start = chars
        .next()
        .ok_or(&parse_pos_err)?
        .to_digit(10)
        .ok_or(&parse_pos_err)? as usize;
    let _ = chars.next().ok_or(&parse_pos_err)?;
    let end = chars
        .next()
        .ok_or(&parse_pos_err)?
        .to_digit(10)
        .ok_or(&parse_pos_err)? as usize;

    let range = std::ops::Range { start, end };

    Ok(vec![range])
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
    fn test_parse_pos_string_empty() {
        // The empty string is an error
        assert!(parse_pos("").is_err());
    }

    #[test]
    fn test_parse_pos_string_zero_error() {
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "0""#);
    }

    #[test]
    fn test_parse_pos_string_zero_one_error() {
        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "0""#);
    }

    #[test]
    fn test_parse_pos_string_leading_plus_error() {
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "+1""#,);
    }

    #[test]
    fn test_parse_pos_leading_plus_range_error() {
        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "+1-2""#,
        );
    }

    #[test]
    fn test_parse_pos_trailing_plus_range_error() {
        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "1-+2""#,
        );
    }

    #[test]
    fn test_parse_pos_alpha_error() {
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);
    }

    #[test]
    fn test_parse_pos_alpha_in_list_error() {
        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);
    }

    #[test]
    fn test_parse_pos_alpha_range_end_error() {
        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "1-a""#);
    }

    #[test]
    fn test_parse_pos_alpha_range_start_error() {
        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a-1""#);
    }

    #[test]
    fn test_parse_pos_dash_only_error() {
        assert!(parse_pos("-").is_err());
    }

    #[test]
    fn test_parse_pos_comma_only_error() {
        assert!(parse_pos(",").is_err());
    }

    #[test]
    fn test_parse_pos_trailing_comma_error() {
        assert!(parse_pos("1,").is_err());
    }

    #[test]
    fn test_parse_pos_trailing_dash_error() {
        assert!(parse_pos("1-").is_err());
    }

    #[test]
    fn test_parse_pos_triple_range_error() {
        assert!(parse_pos("1-1-1").is_err());
    }

    #[test]
    fn test_parse_pos_triple_range_alpha_error() {
        assert!(parse_pos("1-1-a").is_err());
    }

    #[test]
    fn test_parse_pos_equal_range_error() {
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );
    }

    #[test]
    fn test_parse_pos_inverted_range_error() {
        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );
    }

    #[test]
    fn test_parse_pos_single_number() {
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
    }

    #[test]
    fn test_parse_pos_single_number_with_leading_zero() {
        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
    }

    #[test]
    fn test_parse_pos_list() {
        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
    }

    #[test]
    fn test_parse_pos_list_with_leading_zeros() {
        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
    }

    #[test]
    fn test_parse_pos_range() {
        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
    }

    #[test]
    fn test_parse_pos_range_with_leading_zeros() {
        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
    }

    #[test]
    fn test_parse_pos_mixed_list_and_range() {
        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);
    }

    #[test]
    fn test_parse_pos_large_values() {
        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }
}

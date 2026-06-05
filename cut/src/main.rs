#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use anyhow::{anyhow, bail, Result};
use clap;
use clap::{Args, Parser};
use csv::StringRecord;
use std::io::Read;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    ops::Range,
};

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
    fields: Option<PositionList>,
    #[arg(short, long, value_parser=parse_pos)]
    bytes: Option<PositionList>,
    #[arg(short, long, value_parser=parse_pos)]
    chars: Option<PositionList>,
}

fn main() {
    let args = Cli::parse();
    if let Err(e) = run(args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

/// Checks if value is specifically one byte
fn is_byte(delim_str: &str) -> Result<String, String> {
    let delim_err = format!("--delim \"{delim_str}\" must be a single byte");

    let mut delim_chars = delim_str.chars();
    if delim_chars.clone().count() != 1 {
        return Err(delim_err);
    }
    let delim = delim_chars.next().ok_or(&delim_err)?;

    if delim.is_ascii() {
        Ok(delim.to_string())
    } else {
        Err(delim_err)
    }
}

fn parse_pos(range: &str) -> Result<PositionList, String> {
    let parse_err_str = &format!("illegal list value: \"{}\"", range.to_string());

    if range.is_empty() {
        return Err(parse_err_str.to_string());
    }
    if !range.starts_with(|c: char| c.is_digit(10)) {
        return Err(parse_err_str.to_string());
    }

    // Parse comma-separated values, each can be a single number or a range
    range.split(',').try_fold(Vec::new(), |mut acc, part| {
        if part.is_empty() {
            return Err(parse_err_str.to_string());
        }

        // Check if this part is a range (contains dash)
        if let Some(dash_pos) = part.find('-') {
            // Make sure the dash is not at the start or end
            if dash_pos == 0 || dash_pos == part.len() - 1 {
                return Err(parse_err_str.to_string());
            }

            // Split on first dash only
            let (start_str, end_str) = part.split_at(dash_pos);
            let end_str = &end_str[1..]; // Skip the dash

            if start_str.contains("+") || end_str.contains("+") {
                return Err(parse_err_str.to_string());
            }

            // Parse start and end
            let start = start_str
                .parse::<usize>()
                .map_err(|_| parse_err_str.to_string())?;
            let end = end_str
                .parse::<usize>()
                .map_err(|_| parse_err_str.to_string())?;

            // Validate: start must be > 0 and start < end
            if start == 0 {
                return Err(format!("illegal list value: \"{start}\""));
            }
            if start >= end {
                return Err(format!(
                    "First number in range ({start}) must be lower than second number ({end})",
                ));
            }

            // Add range: convert 1-based to 0-based
            acc.push((start - 1)..end);
        } else {
            // Single number
            let num = part
                .parse::<usize>()
                .map_err(|_| format!("illegal list value: \"{part}\""))?;

            if num == 0 {
                return Err(parse_err_str.to_string());
            }

            // Add single position: convert 1-based to 0-based
            acc.push((num - 1)..num);
        }

        Ok(acc)
    })
}

fn run(args: Cli) -> Result<()> {
    for filename in &args.files {
        match open(filename) {
            Err(err) => eprintln!("{filename}: {err}"),
            Ok(reader) => {
                for l in reader.lines() {
                    let line = l?;

                    let output = if let Some(pos_list) = &args.input.bytes {
                        extract_bytes(&line, pos_list)
                    } else if let Some(pos_list) = &args.input.chars {
                        extract_chars(&line, pos_list)
                    } else if let Some(pos_list) = &args.input.fields {
                        let mut s = String::new();
                        let mut rdr = csv::Reader::from_reader(line.as_bytes());
                        for result in rdr.records() {
                            let record = result?;
                            let fields = extract_fields(&record, pos_list);
                            s.push_str(&fields.join(" "));
                        }
                        s
                    } else {
                        bail!("No position list found")
                    };

                    println!("{output}");
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

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let mut extracted = String::new();

    // TODO: Get this working for exclusive range
    for range in char_pos {
        let s: String = line
            .char_indices()
            .filter_map(|(i, c)| if range.contains(&i) { Some(c) } else { None })
            .collect();

        extracted.push_str(&s);
    }

    extracted
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let mut extracted: Vec<u8> = Vec::new();

    for range in byte_pos {
        let mut bytes: Vec<u8> = line
            .as_bytes()
            .into_iter()
            .enumerate()
            .filter_map(|e| {
                if range.contains(&e.0) {
                    Some(e.1.to_owned())
                } else {
                    None
                }
            })
            .collect();

        extracted.append(&mut bytes);
    }

    String::from_utf8(extracted).unwrap_or_else(|_| "�".to_string())
}

fn extract_fields(record: &StringRecord, field_pos: &[Range<usize>]) -> Vec<String> {
    let mut fields: Vec<String> = vec![];

    dbg!(record);

    for range in field_pos {
        let field: String = record
            .clone()
            .iter()
            .enumerate()
            .filter_map(|(i, s)| if range.contains(&i) { Some(s) } else { None })
            .collect();

        if !field.is_empty() {
            fields.push(field);
        }
    }

    fields
}

#[cfg(test)]
mod unit_tests {
    use super::{extract_bytes, extract_chars, extract_fields, parse_pos};
    use csv::StringRecord;

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
        dbg!(&res);

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

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }
}

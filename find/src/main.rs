use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser, ValueEnum};
use regex::Regex;
use std::fs::FileType;
use walkdir::WalkDir;

// Have to put this up here because its hard to pass this default into clap
const DEFAULT_ENTRY_TYPE: &[EntryType] = &[EntryType::Dir, EntryType::File, EntryType::Link];

#[derive(Debug, Copy, Clone, Eq, PartialEq, ValueEnum)]
enum EntryType {
    #[value(name = "d")]
    Dir,
    #[value(name = "f")]
    File,
    #[value(name = "l")]
    Link,
}

impl EntryType {
    fn matches(self, file_type: &FileType) -> bool {
        match self {
            EntryType::Dir => file_type.is_dir(),
            EntryType::File => file_type.is_file(),
            EntryType::Link => file_type.is_symlink(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(default_value=".", value_name="PATH", num_args =0.., action=ArgAction::Append)]
    paths: Vec<String>,
    #[arg(short='n', long = "name", default_value=".*", value_parser=Regex::new, value_name="NAME", num_args=0.., action=ArgAction::Append
    )]
    names: Vec<Regex>,
    #[arg(short='t', long="type", value_name="TYPE", num_args=..=3, action=ArgAction::Append)]
    entry_types: Vec<EntryType>,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let Args {
        paths,
        names,
        entry_types,
    } = args;

    let entry_types = if entry_types.is_empty() {
        DEFAULT_ENTRY_TYPE
    } else {
        &entry_types
    };

    for path in paths {
        for res in WalkDir::new(path) {
            let entry = match res {
                Ok(entry) => entry,
                // print out any IO error for an entry and move on
                Err(e) if e.io_error().is_some() => {
                    eprintln!("{e}");
                    continue;
                }
                Err(e) => return Err(e.into()),
            };

            let name = entry
                .file_name()
                .to_str()
                .ok_or_else(|| anyhow!("invalid file string"))?;
            let file_type = entry.file_type();

            let matches_type = entry_types
                .iter()
                .any(|entry_type| entry_type.matches(&file_type));
            let matches_name = names.iter().any(|regex| regex.is_match(name));

            if matches_name && matches_type {
                println!("{}", entry.path().display());
            }
        }
    }

    Ok(())
}

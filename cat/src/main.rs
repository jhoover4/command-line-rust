use clap::Parser;

/// Rust version of `cat`
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input files
    files: Vec<String>,
    #[arg(short, long)]
    number_lines: bool,
    #[arg(short = 'b', long)]
    number_nonblank_lines: bool,
}

fn main() {
    let args = Args::parse();

    println!("{args:#?}");
}

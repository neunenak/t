use std::io::{self, Write};
use std::path::PathBuf;

use clap::Parser;

mod ast;
mod error;
mod interpreter;
mod operators;
mod parser;
mod value;

use interpreter::Context;
use value::{Array, Level, Value};

#[derive(Parser)]
#[command(name = "t")]
#[command(
    about = "T is a concise language for manipulating text, replacing common usage patterns of Unix utilities like grep, sed, cut, awk, sort, and uniq."
)]
struct Cli {
    /// Programme to execute
    prog: String,

    /// Optional files to process
    files: Vec<String>,

    /// Output as JSON
    #[arg(short = 'j', long = "json")]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    let programme = match parser::parse_programme(&cli.prog) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let input = if cli.files.is_empty() {
        Array::from_stdin(Level::Line)
    } else {
        let paths: Vec<PathBuf> = cli.files.iter().map(PathBuf::from).collect();
        Array::from_files(&paths, Level::Line)
    };

    let array = match input {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            std::process::exit(1);
        }
    };

    let ops = interpreter::compile(&programme);
    let mut ctx = Context::new(Value::Array(array));

    if let Err(e) = interpreter::run(&ops, &mut ctx) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let value = ctx.into_value();
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    if cli.json {
        serde_json::to_writer_pretty(&mut handle, &value).expect("JSON serialization failed");
    } else {
        write!(handle, "{}", value).expect("write failed");
    }
    writeln!(handle).expect("write failed");
}

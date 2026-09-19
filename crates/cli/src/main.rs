//! The `lumen` command-line interface.
//!
//! This crate is CLI only: argument parsing, file access, and help rendering. Compiler logic
//! lives in the per-phase crates under `crates/`, and what canonical form is belongs to
//! `lumen-format` and to `docs/specs/formatting.md`.

use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::{Parser, Subcommand};
use lumen_format::format;

/// The Lumen compiler.
#[derive(Parser)]
#[command(version, about, long_about = include_str!("help/lumen.md"))]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Rewrite a source file in canonical form
    #[command(long_about = include_str!("help/fmt.md"))]
    Fmt { file: PathBuf },
    /// Report the first line of a source file that is not in canonical form
    #[command(long_about = include_str!("help/check.md"))]
    Check { file: PathBuf },
}

fn main() {
    let Cli { command } = Cli::parse();
    exit(run(&command).code());
}

fn run(command: &Command) -> Outcome {
    match command {
        Command::Fmt { file } => fmt(file),
        Command::Check { file } => check(file),
    }
}

/// Writes the canonical text of `path` back to it, leaving a canonical file untouched.
fn fmt(path: &Path) -> Outcome {
    let Some(source) = source_of(path) else {
        return Outcome::Unusable;
    };
    match format(&source) {
        Err(error) => refuse(path, &error.message()),
        Ok(canonical) if canonical == source => Outcome::Done,
        Ok(canonical) => rewrite(path, &canonical),
    }
}

/// Reports the first line of `path` that canonical form writes differently.
fn check(path: &Path) -> Outcome {
    let Some(source) = source_of(path) else {
        return Outcome::Unusable;
    };
    match lumen_format::check(&source) {
        Ok(()) => Outcome::Done,
        Err(error) => refuse(path, &error.to_string()),
    }
}

/// What a run amounted to, which is what its exit code says.
enum Outcome {
    Done,
    /// The file is a file, and the compiler will not have it.
    Refused,
    /// The file could not be read or written, which is not about the program in it.
    Unusable,
}

impl Outcome {
    const fn code(&self) -> i32 {
        match self {
            Self::Done => 0,
            Self::Refused => 1,
            Self::Unusable => 2,
        }
    }
}

fn source_of(path: &Path) -> Option<String> {
    match read_to_string(path) {
        Ok(source) => Some(source),
        Err(error) => {
            eprintln!("error: {}: {error}", path.display());
            None
        }
    }
}

fn rewrite(path: &Path, canonical: &str) -> Outcome {
    match write(path, canonical) {
        Ok(()) => Outcome::Done,
        Err(error) => {
            eprintln!("error: {}: {error}", path.display());
            Outcome::Unusable
        }
    }
}

/// Reports why a file will not do, in the voice of `docs/implementation.md` section 8.
fn refuse(path: &Path, message: &str) -> Outcome {
    eprintln!("error: {}: {message}", path.display());
    Outcome::Refused
}

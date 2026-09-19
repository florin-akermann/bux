//! The `lumen` command-line interface.
//!
//! This crate is CLI only: argument parsing, file access, and help rendering. Compiler logic
//! lives in the per-phase crates under `crates/`, what canonical form is belongs to
//! `lumen-format`, and how a refusal reads belongs to `lumen-diagnostics`.

use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::{Parser, Subcommand};
use lumen_diagnostics::{Code, Diagnostic, render};
use lumen_exhaustiveness::check as exhaustive;
use lumen_format::format;
use lumen_parser::parse;
use lumen_resolver::resolve;
use lumen_types::check as inferred;

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
    /// Report the first thing about a source file the compiler will not have
    #[command(long_about = include_str!("help/check.md"))]
    Check { file: PathBuf },
    /// Print the long form of one diagnostic code
    #[command(long_about = include_str!("help/explain.md"))]
    Explain { code: String },
}

fn main() {
    let Cli { command } = Cli::parse();
    exit(run(&command).code());
}

fn run(command: &Command) -> Outcome {
    match command {
        Command::Fmt { file } => fmt(file),
        Command::Check { file } => check(file),
        Command::Explain { code } => explain(code),
    }
}

/// Writes the canonical text of `path` back to it, leaving a canonical file untouched.
fn fmt(path: &Path) -> Outcome {
    let Some(source) = source_of(path) else {
        return Outcome::Unusable;
    };
    match format(&source) {
        Err(error) => refuse(&error.diagnostic(), &source, path),
        Ok(canonical) if canonical == source => Outcome::Done,
        Ok(canonical) => rewrite(path, &canonical),
    }
}

/// Reports the first thing about `path` the compiler will not have.
fn check(path: &Path) -> Outcome {
    let Some(source) = source_of(path) else {
        return Outcome::Unusable;
    };
    match accepted(&source) {
        Ok(()) => Outcome::Done,
        Err(diagnostic) => refuse(&diagnostic, &source, path),
    }
}

/// Every phase the front end has, run in order, stopping at the first refusal.
fn accepted(source: &str) -> Result<(), Diagnostic> {
    lumen_format::check(source).map_err(|error| error.diagnostic())?;
    let program = parse(source).map_err(|error| error.diagnostic())?;
    let resolved = resolve(program).map_err(|error| error.diagnostic())?;
    let typed = inferred(resolved).map_err(|error| error.diagnostic())?;
    exhaustive(&typed).map_err(|error| error.diagnostic())?;
    Ok(())
}

/// Prints what one diagnostic code means, at more length than its `help:` line has room for.
fn explain(written: &str) -> Outcome {
    let Some(code) = Code::written_as(written) else {
        eprintln!("error: there is no diagnostic {written}");
        return Outcome::Unusable;
    };
    print!("{}", code.explanation());
    Outcome::Done
}

/// What a run amounted to, which is what its exit code says.
enum Outcome {
    Done,
    /// The compiler will not have what it was given.
    Refused,
    /// The command could not do its job at all, which is not about any program.
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

/// Reports why a file will not do, in the layout of `docs/specs/diagnostics.md`.
fn refuse(diagnostic: &Diagnostic, source: &str, path: &Path) -> Outcome {
    eprint!(
        "{}",
        render(diagnostic, source, &path.display().to_string())
    );
    Outcome::Refused
}

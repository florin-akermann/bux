//! The `lumen` command-line interface.
//!
//! This crate is CLI only: argument parsing and help rendering. Compiler logic lives in the
//! per-phase crates under `crates/`.

use clap::Parser;

/// The Lumen compiler.
#[derive(Parser)]
#[command(version, about, long_about = include_str!("help/lumen.md"))]
struct Cli {}

fn main() {
    let Cli {} = Cli::parse();
}

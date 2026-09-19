//! The Lumen parser: source text to an untyped abstract syntax tree.
//!
//! Parsing is fallible and stops at the first error, which carries the span of the token that
//! failed. The parser decides nothing a later phase can decide better: it knows no definitions,
//! so it never asks whether a name exists or what it names. The grammar it accepts is specified
//! in `docs/specs/grammar.md`.

mod control;
mod cursor;
mod error;
mod expr;
mod item;
mod list;
mod literal;
mod pattern;
mod record;
mod stmt;
mod type_ref;

pub use error::ParseError;

use cursor::Cursor;
use lumen_ast::Program;

/// Parses `source` into the program it spells.
///
/// # Errors
///
/// Returns the first place the source does not fit the grammar.
pub fn parse(source: &str) -> Result<Program, ParseError> {
    item::program(&mut Cursor::new(source))
}

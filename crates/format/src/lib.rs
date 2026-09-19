//! The one canonical form of a Lumen program, and the gate that requires it.
//!
//! `docs/design.md` section 13 makes source that is not in canonical form a compile error, and
//! this crate is where that form is defined: [`format`] writes it, and [`check`] reports the
//! first line of a file that departs from it. The rules are stated in
//! `docs/specs/formatting.md`.
//!
//! Formatting preserves meaning. `parse(format(source))` equals `parse(source)` for every source
//! that parses, so no file ever changes what it says by being formatted.

mod control;
mod expr;
mod item;
mod literal;
mod operand;
mod pattern;
mod printer;
mod stmt;
mod type_ref;

use std::fmt;
use std::num::NonZeroUsize;

use lumen_parser::{ParseError, parse};

use crate::printer::Printer;

/// Whether `source` is in canonical form already.
///
/// # Errors
///
/// Returns the parse error when `source` is not a program, and the first deviation when it is a
/// program that is not written canonically.
pub fn check(source: &str) -> Result<(), CheckError> {
    let canonical = format(source).map_err(CheckError::Parse)?;
    if canonical == source {
        return Ok(());
    }
    Err(CheckError::NotCanonical(first_deviation(
        source, &canonical,
    )))
}

/// The canonical text of `source`.
///
/// # Errors
///
/// Returns the parse error when `source` is not a program.
pub fn format(source: &str) -> Result<String, ParseError> {
    let program = parse(source)?;
    let mut printer = Printer::new(source);
    item::program(&mut printer, &program, source.len());
    Ok(printer.finish())
}

/// Why `check` refused a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckError {
    Parse(ParseError),
    NotCanonical(Deviation),
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "{}", error.message()),
            Self::NotCanonical(deviation) => write!(f, "{deviation}"),
        }
    }
}

/// Where a file first departs from canonical form.
///
/// There is no fourth case: canonical form never ends with a line the source does not reach,
/// because every line it writes is made of text the source already holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Deviation {
    /// A line whose text is not what canonical form writes there.
    Line {
        number: LineNumber,
        found: String,
        canonical: String,
    },
    /// A line the source has and canonical form does not.
    Extra { number: LineNumber, found: String },
    /// Every line matches, and only the way the file ends does not.
    Ending,
}

impl fmt::Display for Deviation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Line {
                number,
                found,
                canonical,
            } => write!(
                f,
                "line {number} is not in canonical form\n  found:     {found}\n  canonical: {canonical}"
            ),
            Self::Extra { number, found } => {
                write!(f, "line {number} is not part of canonical form: {found}")
            }
            Self::Ending => write!(f, "the file does not end the way canonical form ends it"),
        }
    }
}

/// The first line the two texts disagree on, or the ending when every line agrees.
fn first_deviation(source: &str, canonical: &str) -> Deviation {
    let wanted: Vec<&str> = lines_of(canonical).collect();
    for (index, found) in lines_of(source).enumerate() {
        let number = LineNumber::of(index);
        match wanted.get(index) {
            None => {
                return Deviation::Extra {
                    number,
                    found: found.to_owned(),
                };
            }
            Some(line) if *line == found => {}
            Some(line) => {
                return Deviation::Line {
                    number,
                    found: found.to_owned(),
                    canonical: (*line).to_owned(),
                };
            }
        }
    }
    Deviation::Ending
}

/// The lines of a text, keeping a carriage return canonical form would not write.
///
/// `str::lines` takes a `\r` off the end of a line, which would let a file written with Windows
/// line endings match canonical form line for line and be reported for its ending instead.
fn lines_of(text: &str) -> impl Iterator<Item = &str> {
    text.split_inclusive('\n')
        .map(|line| line.strip_suffix('\n').unwrap_or(line))
}

/// A one-based line number, as an editor counts lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineNumber(NonZeroUsize);

impl LineNumber {
    /// The line an index into a list of lines names.
    fn of(index: usize) -> Self {
        Self(NonZeroUsize::new(index + 1).expect("one more than an index is never zero"))
    }

    /// The number, as it is written.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

impl fmt::Display for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

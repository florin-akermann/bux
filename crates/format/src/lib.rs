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
mod naming;
mod operand;
mod order;
mod pattern;
mod printer;
mod stmt;
mod type_ref;

use std::fmt;

use lumen_ast::TypeRef;
use lumen_ast::{DeriveDeclaration, InstanceDeclaration, TraitDeclaration, TypeDeclaration};
use lumen_diagnostics::{Code, Diagnostic, Fix};
use lumen_lexer::Span;
use lumen_parser::{ParseError, parse};

pub use crate::naming::{Kind, Misspelling};
pub use crate::order::OutOfOrder;
use crate::printer::Printer;

/// Whether `source` is in canonical form already.
///
/// # Errors
///
/// Returns the parse error when `source` is not a program, the first deviation when it is a
/// program that is not written canonically, and the first import that is out of place.
pub fn check(source: &str) -> Result<(), CheckError> {
    let program = parse(source).map_err(CheckError::Parse)?;
    let mut printer = Printer::new(source);
    item::program(&mut printer, &program, source.len());
    let canonical = printer.finish();
    if canonical != source {
        return Err(CheckError::NotCanonical {
            deviation: first_deviation(source, &canonical),
            fix: Fix::new(Span::new(0, source.len()), canonical),
        });
    }
    if let Some(out) = order::out_of_order(&program) {
        return Err(CheckError::OutOfOrder(out));
    }
    naming::misspelled(&program).map_or(Ok(()), |name| Err(CheckError::Misspelled(name)))
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

/// The canonical text of `declared`, as a file holds it, and holding no comment.
///
/// A type declaration is all surface, so this is also how `lumen api` prints one: one printer
/// writes both, and a page can therefore never drift from the form a file is held to.
#[must_use]
pub fn type_declaration(declared: &TypeDeclaration) -> String {
    let mut printer = Printer::without_comments();
    item::type_declaration(&mut printer, declared);
    printer.finish()
}

/// The canonical text of `declared`, as a file holds it, and holding no comment.
///
/// A trait is all surface, as a type declaration is: it declares signatures and no body, so this
/// is how `lumen api` prints one too.
#[must_use]
pub fn trait_declaration(declared: &TraitDeclaration) -> String {
    let mut printer = Printer::without_comments();
    item::trait_declaration(&mut printer, declared);
    printer.finish()
}

/// `instance Eq<Point>`, which is what an instance puts on an API page.
///
/// What the instance writes is what its trait already declares, so the page states that the type
/// has the trait and leaves the bodies where every other body is left.
#[must_use]
pub fn instance_head(declared: &InstanceDeclaration) -> String {
    format!(
        "instance {}<{}>\n",
        declared.trait_name.text, declared.for_type.text
    )
}

/// `instance Eq<User>`, which is what a derive puts on an API page.
///
/// A derived instance is an instance, and `docs/specs/derive.md` leaves nothing able to tell the
/// two apart, so the page states the trait the type has and not how it came by it.
#[must_use]
pub fn derived_head(declared: &DeriveDeclaration) -> String {
    let [named] = declared.traits.as_slice() else {
        unreachable!("`Eq` is the one trait a type derives, so a derive that resolved names one")
    };
    format!("instance {}<{}>\n", named.text, declared.for_type.text)
}

/// The canonical text of a type as it is written, which a constraint on a page is stated with.
#[must_use]
pub fn written_type(written: &TypeRef) -> String {
    let mut printer = Printer::without_comments();
    type_ref::type_ref(&mut printer, written);
    printer.finish()
}

/// Why `check` refused a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckError {
    Parse(ParseError),
    NotCanonical {
        /// Where the file first departs from canonical form, which is what the reader is shown.
        deviation: Deviation,
        /// The edit that answers it: the canonical text, in place of the whole file.
        fix: Fix,
    },
    /// An import is written somewhere canonical form does not put it.
    OutOfOrder(OutOfOrder),
    /// A declared name is spelled some way other than the one canonical form spells it.
    Misspelled(Misspelling),
}

impl CheckError {
    /// This refusal as the diagnostic the reader is shown.
    #[must_use]
    pub fn diagnostic(&self) -> Diagnostic {
        match self {
            Self::Parse(error) => error.diagnostic(),
            Self::NotCanonical { deviation, fix } => Diagnostic::new(
                Code::NotCanonical,
                deviation.to_string(),
                deviation.span(),
                Some(deviation.help()),
            )
            .fixed_by(fix.clone()),
            Self::OutOfOrder(out) => Diagnostic::new(
                Code::ImportOutOfOrder,
                out.to_string(),
                out.span(),
                Some(out.help()),
            ),
            Self::Misspelled(name) => Diagnostic::new(
                name.code(),
                name.to_string(),
                name.span(),
                Some(name.help()),
            ),
        }
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "{}", error.message()),
            Self::NotCanonical { deviation, .. } => write!(f, "{deviation}"),
            Self::OutOfOrder(out) => write!(f, "{out}"),
            Self::Misspelled(name) => write!(f, "{name}"),
        }
    }
}

/// Where a file first departs from canonical form.
///
/// There is no fourth case: the source can only run out of lines before canonical form does when
/// it does not end the way canonical form ends it, because every line canonical form writes is
/// made of text the source already holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Deviation {
    /// A line whose text is not what canonical form writes there.
    Line { span: Span, canonical: String },
    /// A line the source has and canonical form does not.
    Extra { span: Span },
    /// Every line matches, and only the way the file ends does not.
    Ending { span: Span },
}

impl Deviation {
    /// The source the deviation points at, which is the line it is about.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Line { span, .. } | Self::Extra { span } | Self::Ending { span } => *span,
        }
    }

    /// What to do about it, which for a line canonical form rewrites is the text it writes.
    fn help(&self) -> String {
        match self {
            Self::Line { canonical, .. } => format!("canonical form writes `{canonical}`"),
            Self::Extra { .. } => "run `lumen fmt` to take this line out".to_owned(),
            Self::Ending { .. } => "end the file with a newline".to_owned(),
        }
    }
}

impl fmt::Display for Deviation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Line { .. } => write!(f, "this line is not in canonical form"),
            Self::Extra { .. } => write!(f, "canonical form does not write this line"),
            Self::Ending { .. } => {
                write!(f, "this file does not end the way canonical form ends it")
            }
        }
    }
}

/// The first line the two texts disagree on, or the ending when every line agrees.
fn first_deviation(source: &str, canonical: &str) -> Deviation {
    let wanted: Vec<&str> = lines_of(canonical).map(|line| line.text).collect();
    let mut last = Line { at: 0, text: "" };
    for (index, line) in lines_of(source).enumerate() {
        last = line;
        match wanted.get(index) {
            None => return Deviation::Extra { span: line.span() },
            Some(text) if *text == line.text => {}
            Some(text) => {
                return Deviation::Line {
                    span: line.span(),
                    canonical: (*text).to_owned(),
                };
            }
        }
    }
    Deviation::Ending { span: last.span() }
}

/// The lines of a text, keeping a carriage return canonical form would not write.
///
/// `str::lines` takes a `\r` off the end of a line, which would let a file written with Windows
/// line endings match canonical form line for line and be reported for its ending instead.
fn lines_of(text: &str) -> impl Iterator<Item = Line<'_>> {
    let mut at = 0;
    text.split_inclusive('\n').map(move |line| {
        let starts = at;
        at += line.len();
        Line {
            at: starts,
            text: line.strip_suffix('\n').unwrap_or(line),
        }
    })
}

/// One line of a text, and the byte offset it begins at.
#[derive(Clone, Copy)]
struct Line<'a> {
    at: usize,
    text: &'a str,
}

impl Line<'_> {
    const fn span(self) -> Span {
        Span::new(self.at, self.text.len())
    }
}

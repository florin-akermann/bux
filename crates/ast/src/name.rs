//! Identifier occurrences.

use crate::Span;

/// One occurrence of an identifier, with the text it was spelled with.
///
/// A name is only a word here. Whether it names a function, a type, a variant, or a local is
/// name resolution's answer, and the parser never guesses at it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

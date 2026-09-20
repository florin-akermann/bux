//! Patterns, as written in the arms of a `match`.

use crate::{Name, Path, Span};

/// A pattern, with the span of the source it was parsed from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

/// The forms a pattern is written in, which `docs/specs/patterns.md` states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternKind {
    /// A name: it binds the value, or it names a variant that carries nothing.
    ///
    /// `Pending` and `reason` are the same shape here; name resolution decides which is which.
    /// One reached through a module is always the variant, because nothing binds a dotted name.
    Name(Path),
    /// `Failed(reason)`, a variant matched on what it carries positionally.
    Tuple {
        path: Path,
        elements: Vec<Pattern>,
    },
    /// `Authorized { authorization_id }`, a variant matched on the fields it names.
    Record {
        path: Path,
        fields: Vec<Name>,
    },
    Integer(i64),
    String(String),
    Bool(bool),
    /// `_`: it matches every value and binds nothing, which is what a name cannot say.
    Wildcard,
    /// `Pending | Running`, which matches what any one of its alternatives matches.
    ///
    /// Every alternative binds nothing, so the arm reads the same whichever one answered.
    Or(Vec<Pattern>),
}

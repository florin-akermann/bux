//! Patterns, as written in the arms of a `match`.

use crate::{Name, Span};

/// A pattern, with the span of the source it was parsed from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

/// The pattern forms of version 0.1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternKind {
    /// A bare name: it binds the value, or it names a variant that carries nothing.
    ///
    /// `Pending` and `reason` are the same shape here; name resolution decides which is which.
    Name(Name),
    /// `Failed(reason)`, a variant matched on what it carries positionally.
    Tuple {
        name: Name,
        elements: Vec<Pattern>,
    },
    /// `Authorized { authorization_id }`, a variant matched on the fields it names.
    Record {
        name: Name,
        fields: Vec<Name>,
    },
    Integer(i64),
    String(String),
    Bool(bool),
}

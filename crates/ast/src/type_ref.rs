//! Types as they are written in source.

use crate::{Name, Span};

/// A type written in source, such as `Int`, `List<User>`, or `()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeRef {
    pub kind: TypeRefKind,
    pub span: Span,
}

/// The two shapes a written type takes in version 0.1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeRefKind {
    /// A name, with the type arguments applied to it; `Int` has none, `List<User>` has one.
    Named { name: Name, arguments: Vec<TypeRef> },
    /// `()`, the type of a function that returns nothing interesting.
    Unit,
}

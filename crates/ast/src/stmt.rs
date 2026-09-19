//! Blocks, and the statements inside them.

use crate::{Expr, Name, Span};

/// A `{ … }` body: its statements, in source order.
///
/// A block's value is the value of its last statement when that statement is an expression;
/// which blocks need one is type inference's question, not the parser's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// One statement of a block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

/// The statement forms of version 0.1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatementKind {
    /// `total := 0` binds immutably; `var total = 0` binds mutably.
    Binding {
        mutability: Mutability,
        name: Name,
        value: Expr,
    },
    /// `total = 1` or `total += 1`.
    Assign {
        target: Expr,
        operator: AssignOperator,
        value: Expr,
    },
    Return(Option<Expr>),
    Break,
    Continue,
    For(Box<ForLoop>),
    /// An expression evaluated for its value or its effect, including `if` and `match`.
    Expr(Expr),
}

/// Whether a binding may be assigned to afterwards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

/// Whether an assignment replaces the value or adds to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignOperator {
    Set,
    Add,
}

/// `for user in users { … }`, `for total < limit { … }`, or a bare `for { … }`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForLoop {
    pub header: ForHeader,
    pub body: Block,
}

/// What a `for` loops over, if anything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForHeader {
    /// A bare `for`: it runs until a `break` or a `return`.
    Forever,
    /// `for <condition>`: it runs while the condition holds.
    While(Expr),
    /// `for <binding> in <iterable>`.
    In { binding: Name, iterable: Expr },
}

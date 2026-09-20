//! Expressions.

use crate::{Block, Name, Path, Pattern, Span};

/// An expression, with the span of the source it was parsed from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

/// What a call passes: the values in order, or each value with the parameter it is passed for.
///
/// A call names all of its arguments or none of them, which `docs/specs/arguments.md` states, so
/// the two are separate shapes rather than one name that may or may not be written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arguments {
    /// `rename(old, new)`, whose arguments the parameter types hold apart.
    Positional(Vec<Expr>),
    /// `rename(from: old, to: new)`, whose arguments the names hold apart.
    Named(Vec<NamedArgument>),
}

impl Arguments {
    /// The values passed, in the order they are written.
    #[must_use]
    pub fn values(&self) -> Vec<&Expr> {
        match self {
            Self::Positional(values) => values.iter().collect(),
            Self::Named(written) => written.iter().map(|one| &one.value).collect(),
        }
    }
}

/// One argument written with the parameter it is passed for, as `from: old`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedArgument {
    pub name: Name,
    pub value: Expr,
}

impl NamedArgument {
    /// The source the whole argument covers, from its name through its value.
    #[must_use]
    pub fn span(&self) -> Span {
        let start = self.name.span.start();
        Span::new(start, self.value.span.end() - start)
    }
}

/// The expression forms of version 0.1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExprKind {
    Name(Name),
    /// A decoded integer literal; a `-` before the digits belongs to it, so `-7` is one of these.
    Integer(i64),
    /// A string literal with its escapes decoded.
    String(String),
    Bool(bool),
    /// `()`.
    Unit,
    Unary {
        operator: UnaryOperator,
        operand: Box<Expr>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Arguments,
    },
    /// `user.name`, which is also how a method is reached before it is called.
    Field {
        receiver: Box<Expr>,
        name: Name,
    },
    /// `find_user(id)?`, which propagates an error.
    Try(Box<Expr>),
    /// `[first, second]`, which builds a list of what it writes; `[]` builds one holding nothing.
    List(Vec<Expr>),
    /// `User { id: id }` building a record, or `user { name: "Bob" }` updating one.
    ///
    /// Which of the two it is depends on what `base` names, so name resolution decides.
    Record {
        base: Path,
        fields: Vec<FieldValue>,
    },
    If(Box<IfExpr>),
    Match(Box<MatchExpr>),
}

/// `name: value`, one field of a record literal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldValue {
    pub name: Name,
    pub value: Expr,
    pub span: Span,
}

/// The prefix operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
    Negate,
}

/// The infix operators, each left-associative except the comparisons, which do not chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Or,
    And,
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

/// `if a { … } else if b { … } else { … }`.
///
/// The chain is flat rather than nested, so every branch is a node of its own with its own
/// span, and an `else if` that is not an `if` cannot be written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfExpr {
    /// Never empty: the first branch is the `if`, and each one after it is an `else if`.
    pub branches: Vec<Branch>,
    /// The block of a final bare `else`.
    pub otherwise: Option<Block>,
}

/// One `condition { … }` of an `if` chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch {
    pub condition: Expr,
    pub block: Block,
    pub span: Span,
}

/// `match payment { … }`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchExpr {
    pub scrutinee: Expr,
    pub arms: Vec<MatchArm>,
}

/// `Failed(reason) => "Failed: " + reason`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

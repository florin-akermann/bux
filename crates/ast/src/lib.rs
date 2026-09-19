//! The untyped abstract syntax tree of a Lumen source file.
//!
//! Every node carries the [`Span`] of the source it was parsed from, and nothing here is ever
//! mutated: a later phase produces a new representation rather than annotating this one. The
//! tree knows no types and no definitions, so `User { id: id }` and `user { name: "Bob" }` are
//! the same node until name resolution tells them apart. The shape is specified in
//! `docs/specs/grammar.md`.

mod expr;
mod item;
mod name;
mod pattern;
mod stmt;
mod type_ref;

pub use expr::{BinaryOperator, Branch, Expr, ExprKind, FieldValue, IfExpr, UnaryOperator};
pub use expr::{MatchArm, MatchExpr};
pub use item::{Function, Import, Item, Parameter, Program, RecordField, TypeDeclaration};
pub use item::{TypeDefinition, Variant, VariantPayload};
pub use name::Name;
pub use pattern::{Pattern, PatternKind};
pub use stmt::{AssignOperator, Block, ForHeader, ForLoop, Mutability, Statement, StatementKind};
pub use type_ref::{TypeRef, TypeRefKind};

/// A span means the same thing in every phase, so the lexer's is the one every node carries.
pub use lumen_lexer::Span;

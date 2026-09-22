//! The untyped abstract syntax tree of a Lumen source file.
//!
//! Every node carries the [`Span`] of the source it was parsed from, and nothing here is ever
//! mutated: a later phase produces a new representation rather than annotating this one. The
//! tree knows no types and no definitions, so `User { id: id }` and `user { name: "Bob" }` are
//! the same node until name resolution tells them apart. The shape is specified in
//! `docs/specs/grammar.md`.

mod expr;
mod item;
mod java;
mod name;
mod path;
mod pattern;
mod stmt;
mod type_ref;

pub use expr::{Arguments, BinaryOperator, Branch, Expr, ExprKind, FieldValue, IfExpr};
pub use expr::{MatchArm, MatchExpr};
pub use expr::{NamedArgument, UnaryOperator};
pub use item::{Called, Constraint, DeriveDeclaration, ExternDeclaration, Function, Import};
pub use item::{Gives, InstanceDeclaration, Item, Reaches};
pub use item::{Parameter, Program};
pub use item::{RecordField, Signature, TraitDeclaration, TypeDeclaration, TypeDefinition};
pub use item::{TypeParameter, Variant, VariantPayload};
pub use java::JavaName;
pub use name::Name;
pub use path::Path;
pub use pattern::{Pattern, PatternKind};
pub use stmt::{AssignOperator, Block, ForHeader, ForLoop, Mutability, Statement, StatementKind};
pub use type_ref::{TypeRef, TypeRefKind};

/// A span means the same thing in every phase, so the lexer's is the one every node carries.
pub use lumen_lexer::Span;

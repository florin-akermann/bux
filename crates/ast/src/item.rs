//! The top-level declarations of a source file.

use crate::{Block, Name, Span, TypeRef};

/// One source file: the items it declares, in source order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub items: Vec<Item>,
}

/// A top-level declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Import(Import),
    Type(TypeDeclaration),
    Function(Function),
}

/// `import io`: another module's public names are brought into scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    pub module: Name,
    pub span: Span,
}

/// `type User = { … }` or `type Payment = | Pending | Failed(String)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDeclaration {
    pub name: Name,
    pub parameters: Vec<Name>,
    pub definition: TypeDefinition,
    pub span: Span,
}

/// What a type declaration declares: a record, or one or more variants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeDefinition {
    Record(Vec<RecordField>),
    /// Never empty: `type T =` with nothing after it is a parse error.
    Variants(Vec<Variant>),
}

/// One variant of an algebraic data type, with whatever it carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    pub name: Name,
    pub payload: VariantPayload,
    pub span: Span,
}

/// What a variant carries: nothing, positional types, or named fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariantPayload {
    None,
    /// Never empty: `Failed()` is a parse error.
    Tuple(Vec<TypeRef>),
    Record(Vec<RecordField>),
}

/// `name: Type`, one line of a record body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordField {
    pub name: Name,
    pub type_ref: TypeRef,
    pub span: Span,
}

/// `fn count_active(users: List<User>, limit: Int) -> Int { … }`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    pub name: Name,
    pub type_parameters: Vec<Name>,
    pub parameters: Vec<Parameter>,
    /// The declared result type, when the author wrote one; inference supplies the rest.
    pub result: Option<TypeRef>,
    pub body: Block,
    pub span: Span,
}

/// One parameter of a function, with its type when the author wrote one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter {
    pub name: Name,
    pub type_ref: Option<TypeRef>,
    pub span: Span,
}

//! The top-level declarations of a source file.

use std::slice;

use crate::{Block, Name, Span, TypeRef};

/// One source file: the items it declares, in source order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub items: Vec<Item>,
}

impl Program {
    /// Every function the file writes: the ones it declares, and the ones its instances give.
    ///
    /// An instance's method has a body like any other function's, so every phase that walks a
    /// body walks it too. What tells the two apart is that only a declared function has a name
    /// in scope, which `docs/specs/traits.md` states.
    pub fn functions(&self) -> impl Iterator<Item = &Function> {
        self.items.iter().flat_map(|item| match item {
            Item::Function(function) => slice::from_ref(function),
            Item::Instance(instance) => instance.methods.as_slice(),
            Item::Import(_) | Item::Type(_) | Item::Trait(_) | Item::Derive(_) => &[],
        })
    }
}

/// A top-level declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Import(Import),
    Type(TypeDeclaration),
    Trait(TraitDeclaration),
    Instance(InstanceDeclaration),
    Derive(DeriveDeclaration),
    Function(Function),
}

/// `derive Eq for User`: the traits the compiler writes the instances of, and the type it writes
/// them for.
///
/// `docs/specs/derive.md` states which traits a type derives and what each one writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeriveDeclaration {
    /// Never empty: a derive naming no trait is a parse error.
    pub traits: Vec<Name>,
    pub for_type: Name,
    pub span: Span,
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

/// `trait Eq<T> { fn equals(a: T, b: T) -> Bool }`.
///
/// A trait declares one type parameter, which every trait `docs/design.md` section 8 names asks
/// for, and one or more method signatures written over it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitDeclaration {
    pub name: Name,
    pub parameter: Name,
    /// Never empty: a trait declaring no method is a parse error.
    pub methods: Vec<Signature>,
    pub span: Span,
}

/// One method a trait declares: a function's first line, with no body to go with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    pub name: Name,
    pub parameters: Vec<Parameter>,
    /// The declared result type, when the author wrote one; inference supplies the rest.
    pub result: Option<TypeRef>,
    pub span: Span,
}

/// `instance Eq<Point> { fn equals(a: Point, b: Point) -> Bool { … } }`.
///
/// The type an instance is for is written by name and without arguments, which
/// `docs/specs/traits.md` states, so `Eq<List<Point>>` is a parse error rather than a refusal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceDeclaration {
    pub trait_name: Name,
    pub for_type: Name,
    /// Never empty: an instance writing no method is a parse error.
    pub methods: Vec<Function>,
    pub span: Span,
}

/// `fn count_active(users: List<User>, limit: Int) -> Int { … }`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    pub name: Name,
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<Parameter>,
    /// The declared result type, when the author wrote one; inference supplies the rest.
    pub result: Option<TypeRef>,
    pub body: Block,
    pub span: Span,
}

/// `T`, or `T: Eq<T>` — a type parameter a function declares, with what it may be asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeParameter {
    pub name: Name,
    pub constraint: Option<Constraint>,
}

/// `Eq<T>`: the trait a type parameter is constrained by, written over what it constrains.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constraint {
    pub name: Name,
    pub argument: TypeRef,
    pub span: Span,
}

/// One parameter of a function, with its type when the author wrote one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter {
    pub name: Name,
    pub type_ref: Option<TypeRef>,
    pub span: Span,
}

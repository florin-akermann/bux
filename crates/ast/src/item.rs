//! The top-level declarations of a source file.

use std::slice;

use crate::{Block, JavaName, Name, Span, TypeRef};

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
            Item::Import(_)
            | Item::Type(_)
            | Item::Trait(_)
            | Item::Derive(_)
            | Item::Extern(_) => &[],
        })
    }

    /// Every extern the file declares, in the order it writes them.
    ///
    /// An extern is a function with no body: nothing walks it, and everything that asks what the
    /// module offers, or what a name is bound to, reads it exactly as it reads a function.
    pub fn externs(&self) -> impl Iterator<Item = &ExternDeclaration> {
        self.items.iter().filter_map(|item| match item {
            Item::Extern(declaration) => Some(declaration),
            Item::Import(_)
            | Item::Type(_)
            | Item::Trait(_)
            | Item::Instance(_)
            | Item::Derive(_)
            | Item::Function(_) => None,
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
    Extern(ExternDeclaration),
}

/// `extern static read_string(path: Path) -> String = "java.nio.file.Files.readString"`.
///
/// One member of one Java class, under the Lumen name and signature this declaration gives it.
/// `docs/specs/interop.md` states what each kind reaches and what crosses the boundary. There is
/// no body: what the member does is the member's, and the declaration only says how to reach it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternDeclaration {
    pub name: Name,
    pub reaches: Reaches,
    /// What the member's own descriptor gives back, which the result alone does not say.
    pub gives: Gives,
    /// Empty for a `field`, and never empty for a `method`, whose receiver is the first of them.
    pub parameters: Vec<ExternParameter>,
    /// Always written: there is no body for inference to read a result off instead.
    pub result: TypeRef,
    pub span: Span,
}

impl ExternDeclaration {
    /// Whether any parameter is narrowed, which is what asks the result to be an `Option`.
    ///
    /// Narrowing a `long` to an `int` loses whatever does not fit, and nothing in Bux is partial,
    /// so an argument outside the `int` range is the `None` this result carries.
    #[must_use]
    pub fn narrows(&self) -> bool {
        self.parameters
            .iter()
            .any(|parameter| parameter.takes == Takes::AnInt)
    }
}

/// One parameter of an `extern`: the parameter itself, and what the member takes it as.
///
/// `docs/specs/interop.md` states the narrowing. Every other declaration writes a parameter
/// alone, because no other declaration reaches a member whose descriptor says something else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternParameter {
    /// The parameter as any signature writes it: its name, and the Lumen type written after it.
    pub declared: Parameter,
    /// What the member's own descriptor takes, which the declared type alone does not say.
    pub takes: Takes,
}

/// Which width the member's own descriptor takes, where two of them answer to one Lumen type.
///
/// `Int` compiles to a `long` and a great many Java members take an `int` instead: `String.charAt`
/// and `String.substring` each do. `docs/specs/interop.md` states the narrowing and what it asks
/// of the result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Takes {
    /// The descriptor the parameter compiles to, which a parameter writing no width says.
    WhatTheParameterIs,
    /// `extern method char at(text: String, int index: Int)`: an `int`, narrowed from the `Int`.
    AnInt,
}

impl Takes {
    /// The word written before the name, which a parameter taken as it is written writes none of.
    #[must_use]
    pub const fn written(self) -> Option<&'static str> {
        match self {
            Self::WhatTheParameterIs => None,
            Self::AnInt => Some(Gives::AN_INT),
        }
    }
}

/// Which width the member's own descriptor gives back, where three of them answer to one Lumen
/// type.
///
/// `Int` compiles to a `long` and a great many Java members give back an `int` or a `char`
/// instead. Which of the three a member gives is written in that member's class file, and the
/// compiler reads none, so the declaration says it. `docs/specs/interop.md` states the widening
/// and what it refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gives {
    /// The descriptor the result compiles to, which is what a declaration writing no width says.
    WhatTheResultIs,
    /// `extern method int length(text: String) -> Int`: an `int`, widened to the `Int` declared.
    AnInt,
    /// `extern method char at(text: String, int index: Int)`: a `char`, widened the same way.
    AChar,
}

impl Gives {
    /// The word that says the member's own descriptor holds an `int`, which is a name elsewhere.
    pub const AN_INT: &'static str = "int";

    /// The word that says the member's own descriptor gives a `char`, which is a name elsewhere.
    pub const A_CHAR: &'static str = "char";

    /// The width `word` spells, where it spells one of the two.
    #[must_use]
    pub fn spelled(word: &str) -> Option<Self> {
        match word {
            Self::AN_INT => Some(Self::AnInt),
            Self::A_CHAR => Some(Self::AChar),
            _ => None,
        }
    }

    /// The word written before the name, which a declaration giving what its result is writes none
    /// of.
    #[must_use]
    pub const fn written(self) -> Option<&'static str> {
        match self {
            Self::WhatTheResultIs => None,
            Self::AnInt => Some(Self::AN_INT),
            Self::AChar => Some(Self::A_CHAR),
        }
    }
}

/// Which kind of member an `extern` reaches, and the Java name that says which one.
///
/// A member with no receiver is named on its class, because nothing else says which class it is
/// on. An instance method is named alone, and a constructor is named by nothing at all: the
/// receiver's type and the result already say. `docs/specs/interop.md` states why that is not
/// left for a check to report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reaches {
    /// `= "java.lang.System.out"`: a static field, read by a call of no arguments.
    Field(JavaName),
    /// `= "java.nio.file.Files.readString"`: a static method.
    Static(JavaName),
    /// `= "toPath"`: an instance method, on the class the first parameter's type is.
    Method(JavaName),
    /// A constructor of the class the result is.
    New,
}

impl Reaches {
    /// The word written after `extern`, which is what says which member it reaches.
    #[must_use]
    pub const fn written(&self) -> &'static str {
        match self {
            Self::Field(_) => "field",
            Self::Static(_) => "static",
            Self::Method(_) => "method",
            Self::New => "new",
        }
    }

    /// The Java name it states, which a constructor states none of.
    #[must_use]
    pub const fn named(&self) -> Option<&JavaName> {
        match self {
            Self::Field(named) | Self::Static(named) | Self::Method(named) => Some(named),
            Self::New => None,
        }
    }
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
    /// `extern type File = "java.io.File"`: the Java class a value of this type is held as.
    ///
    /// It declares no field and no variant, so nothing reads what it holds and nothing matches
    /// on it. `docs/specs/interop.md` states what a program may do with one.
    Foreign {
        class: JavaName,
        /// Which of the two kinds of class it is, which says how a method of it is called.
        called: Called,
    },
}

/// Which kind of class an `extern type` names, where the two are called in two different ways.
///
/// The JVM calls an instance method of a class one way and a method of an interface another.
/// Which of the two a Java name is is written in that name's class file, and the compiler reads
/// none, so the declaration says it. `docs/specs/interop.md` states what the word changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Called {
    /// `extern type File = "java.io.File"`: a class, whose method is called as a class's.
    AsAClass,
    /// `extern type interface Path = "java.nio.file.Path"`: an interface, called as one.
    AsAnInterface,
}

impl Called {
    /// The word a declaration writes before its name to say the class is an interface, which is
    /// an ordinary name everywhere else.
    pub const INTERFACE: &'static str = "interface";

    /// The word written before the name, which a declaration naming a class writes none of.
    #[must_use]
    pub const fn written(self) -> Option<&'static str> {
        match self {
            Self::AsAClass => None,
            Self::AsAnInterface => Some(Self::INTERFACE),
        }
    }
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
/// The type an instance is for is written by name, and its arguments are the type parameters the
/// instance declares: `instance<T: Eq<T>> Eq<List<T>>`, which `docs/specs/traits.md` states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceDeclaration {
    pub trait_name: Name,
    /// `<T: Eq<T>>` after the keyword, which is empty for an instance of a whole type.
    pub type_parameters: Vec<TypeParameter>,
    pub for_type: Name,
    /// The arguments the type is written with, each naming one of the parameters above.
    pub arguments: Vec<Name>,
    /// Never empty: an instance writing no method is a parse error.
    ///
    /// Each one carries the instance's own type parameters, because the method is generic in
    /// them: `is_equal` of `instance<T: Eq<T>> Eq<List<T>>` is written once per `T` it is used
    /// at, exactly as a generic function is.
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

/// `T`, or `T: Eq<T> + Hash<T>` — a type parameter, with what it may be asked to do.
///
/// The list is empty where the author wrote no constraint, and holds one trait for each the
/// author wrote, in the order they are written; `docs/specs/traits.md` states the form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeParameter {
    pub name: Name,
    pub constraints: Vec<Constraint>,
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

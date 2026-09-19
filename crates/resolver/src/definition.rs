//! What a name points at once it has been resolved.

use lumen_ast::Span;

/// The definition one occurrence of a name means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Definition {
    pub kind: DefinitionKind,
    pub origin: Origin,
}

/// What kind of thing a name was declared as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionKind {
    /// A module brought in by an `import`.
    Module,
    /// A type declared by a `type` item.
    Type,
    /// A `<T>` of a function or of a type declaration.
    TypeParameter,
    /// A variant, or the name a record type is built with.
    Constructor,
    /// A function declared by an `fn` item.
    Function,
    /// A parameter of a function.
    Parameter,
    /// A binding made inside a body, by `:=`, `var`, a `for … in`, or a pattern.
    Local,
}

/// Where a definition came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// Declared in this module, at the span of the name that declares it.
    Declared(Span),
    /// In scope in every module without being declared, per `docs/specs/modules.md`.
    Prelude,
}

/// Which of the two scopes a name was written in.
///
/// They never mix, which is what lets `type UserId = UserId(Int64)` declare a type and a
/// constructor of one name; `docs/specs/modules.md` states the rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Namespace {
    /// Searched everywhere a value is written.
    Value,
    /// Searched where a type is written.
    Type,
}

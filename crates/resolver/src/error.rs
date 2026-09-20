//! The one error name resolution can fail with.

use std::fmt;

use lumen_ast::{Name, Span};
use lumen_diagnostics::{Code, Diagnostic};

use crate::definition::Namespace;
use crate::scope::Clash;

/// Why resolution failed, and where.
///
/// Resolution stops at the first error, so there is exactly one of these per failed run. The
/// wording is specified in `docs/specs/modules.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolveError {
    kind: ResolveErrorKind,
    span: Span,
}

impl ResolveError {
    /// The failure `name` caused, pointing at where it is written.
    pub(crate) fn at(name: &Name, kind: ResolveErrorKind) -> Self {
        Self {
            kind,
            span: name.span,
        }
    }

    /// The source the error points at.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// This failure as the diagnostic the reader is shown.
    #[must_use]
    pub fn diagnostic(&self) -> Diagnostic {
        Diagnostic::new(
            self.kind.code(),
            self.message(),
            self.span,
            Some(self.help().to_owned()),
        )
    }

    /// The `error:` line, without its prefix.
    #[must_use]
    pub fn message(&self) -> String {
        self.kind.to_string()
    }

    /// The `help:` line, which every one of these has.
    #[must_use]
    pub const fn help(&self) -> &'static str {
        self.kind.help()
    }
}

/// What went wrong, in the words the reader sees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ResolveErrorKind {
    NoValue(String),
    NoType(String),
    DeclaredTwice(String),
    Shadowed(String),
    /// A declaration written above one that uses it, named in that order.
    WrittenAbove {
        declared: String,
        used_by: String,
    },
    /// A function name written as anything but the name of a call.
    NotCalled(String),
    /// A module name written as a value rather than as what a name is reached through.
    NotReachedThrough(String),
    /// A name that is no module written on the left of the dot of a type or of a pattern.
    NotAModule(String),
    /// An assignment naming something other than a `var` binding.
    NotAVariable(String),
    /// A second instance of one trait for one type, named by both.
    InstanceTwice {
        of: String,
        for_type: String,
    },
    /// A method a trait declares that the instance does not write.
    MethodMissing {
        of: String,
        method: String,
    },
    /// A method an instance writes that its trait never declared.
    MethodUndeclared {
        of: String,
        method: String,
    },
    /// A method an instance writes a second body for, which its trait declares once.
    MethodTwice {
        of: String,
        method: String,
    },
    /// Something that is not a trait, written where a trait belongs.
    NotATrait(String),
    /// A trait a derive names that is not one a type derives.
    NotDerivable(String),
    /// A type a derive names that this module does not declare.
    NotDeclaredHere(String),
    /// A trait written where a type belongs.
    TraitAsType(String),
    /// A name that binds, written inside an or-pattern.
    BindsInsideOr(String),
}

impl ResolveErrorKind {
    /// The failure a name missing from `namespace` amounts to.
    pub(crate) fn missing(namespace: Namespace, text: &str) -> Self {
        match namespace {
            Namespace::Value => Self::NoValue(text.to_owned()),
            Namespace::Type => Self::NoType(text.to_owned()),
        }
    }

    /// The failure a declaration written above something using it amounts to.
    pub(crate) fn written_above(declared: &str, used_by: &str) -> Self {
        Self::WrittenAbove {
            declared: declared.to_owned(),
            used_by: used_by.to_owned(),
        }
    }

    /// The failure a [`Clash`] over `text` amounts to.
    pub(crate) fn of(clash: Clash, text: &str) -> Self {
        match clash {
            Clash::Twice => Self::DeclaredTwice(text.to_owned()),
            Clash::Shadowed => Self::Shadowed(text.to_owned()),
        }
    }

    /// The code this failure is refused with, which `docs/specs/modules.md` lists.
    const fn code(&self) -> Code {
        match self {
            Self::NoValue(_) | Self::NoType(_) => Code::UnresolvedName,
            Self::DeclaredTwice(_) => Code::NameDeclaredTwice,
            Self::Shadowed(_) => Code::NameShadowed,
            Self::WrittenAbove { .. } => Code::DefinitionBeforeUse,
            Self::NotCalled(_) | Self::NotReachedThrough(_) => Code::NotAValue,
            Self::NotAModule(_) => Code::NotAModule,
            Self::NotAVariable(_) => Code::NotAVariable,
            Self::InstanceTwice { .. } => Code::InstanceDeclaredTwice,
            Self::MethodMissing { .. }
            | Self::MethodUndeclared { .. }
            | Self::MethodTwice { .. } => Code::InstanceMethods,
            Self::NotATrait(_) => Code::NotATrait,
            Self::NotDerivable(_) | Self::NotDeclaredHere(_) => Code::NotDerivable,
            Self::TraitAsType(_) => Code::TraitAsType,
            Self::BindsInsideOr(_) => Code::BindsInsideOr,
        }
    }

    const fn help(&self) -> &'static str {
        match self {
            Self::NoValue(_) | Self::NoType(_) => {
                "a name is declared in this file, imported, or supplied by the prelude"
            }
            Self::DeclaredTwice(_) => "one name has one definition; rename one of the two",
            Self::Shadowed(_) => "rename the inner one; Lumen never hides a name",
            Self::WrittenAbove { .. } => "a file reads top down: move it below what uses it",
            Self::NotCalled(_) => "version 0.1 reaches a function by calling it; write the call",
            Self::NotReachedThrough(_) => {
                "a module is what a name is reached through, as `io.println` is"
            }
            Self::NotAModule(_) => {
                "a type of another module is reached through the import: write `demo.User`"
            }
            Self::NotAVariable(_) => "mutation is explicit: bind it with `var`, or bind a new name",
            Self::InstanceTwice { .. } => "one trait and one type have one instance; join the two",
            Self::MethodMissing { .. } => "an instance writes a body for every method it declares",
            Self::MethodUndeclared { .. } => "an instance writes the trait's methods and no others",
            Self::MethodTwice { .. } => "one method of a trait gets one body from an instance",
            Self::NotATrait(_) => "a trait is declared with `trait`, and `Eq` is the one supplied",
            Self::NotDerivable(_) => {
                "`Eq`, `Ord`, `Hash`, and `Show` are the traits a type derives; write others by hand"
            }
            Self::NotDeclaredHere(_) => {
                "a derive reads the declaration it names, so it names one this module writes"
            }
            Self::TraitAsType(_) => "name the type, and constrain it with `<T: Eq<T>>` where it is",
            Self::BindsInsideOr(_) => "write one arm for each alternative where one of them binds",
        }
    }
}

impl fmt::Display for ResolveErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoValue(text) => write!(f, "there is nothing named `{text}`"),
            Self::NoType(text) => write!(f, "there is no type named `{text}`"),
            Self::DeclaredTwice(text) => write!(f, "`{text}` is declared twice in this module"),
            Self::Shadowed(text) => write!(f, "`{text}` is already in scope here"),
            Self::WrittenAbove { declared, used_by } => {
                write!(
                    f,
                    "`{declared}` is written above `{used_by}`, which uses it"
                )
            }
            Self::NotCalled(text) => {
                write!(f, "`{text}` is a function, so it is written as a call")
            }
            Self::NotReachedThrough(text) => {
                write!(
                    f,
                    "`{text}` is a module, so a name inside it is what is written"
                )
            }
            Self::NotAModule(text) => {
                write!(
                    f,
                    "`{text}` is a module in neither scope, and a type is reached through one"
                )
            }
            Self::NotAVariable(text) => {
                write!(f, "`{text}` is not a `var`, so it is never assigned to")
            }
            Self::InstanceTwice { of, for_type } => {
                write!(f, "`{of}` already has an instance for `{for_type}`")
            }
            Self::MethodMissing { of, method } => {
                write!(
                    f,
                    "`{of}` declares `{method}`, which this instance does not write"
                )
            }
            Self::MethodUndeclared { of, method } => {
                write!(
                    f,
                    "`{of}` declares no `{method}` for this instance to write"
                )
            }
            Self::MethodTwice { of, method } => {
                write!(
                    f,
                    "`{of}` declares `{method}` once, and this instance writes it twice"
                )
            }
            Self::NotATrait(text) => write!(f, "`{text}` is not a trait"),
            Self::NotDerivable(text) => {
                write!(f, "`{text}` is not a trait a type derives")
            }
            Self::NotDeclaredHere(text) => {
                write!(f, "`{text}` is not a type this module declares")
            }
            Self::TraitAsType(text) => write!(f, "`{text}` is a trait, not a type"),
            Self::BindsInsideOr(text) => {
                write!(
                    f,
                    "`{text}` binds inside an or-pattern, which binds nothing"
                )
            }
        }
    }
}

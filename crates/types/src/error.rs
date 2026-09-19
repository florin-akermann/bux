//! The one error type inference can fail with.

use std::fmt;

use lumen_ast::Span;
use lumen_diagnostics::{Code, Diagnostic};

use crate::types::Type;

/// Why inference failed, and where.
///
/// Inference stops at the first error, so there is exactly one of these per failed run. The
/// wording is specified in `docs/specs/types.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeError {
    kind: Box<TypeErrorKind>,
    span: Span,
}

impl TypeError {
    /// The failure at `span`.
    pub(crate) fn at(span: Span, kind: TypeErrorKind) -> Self {
        Self {
            kind: Box::new(kind),
            span,
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
pub(crate) enum TypeErrorKind {
    Mismatch {
        expected: Type,
        found: Type,
    },
    NotAddable(Type),
    WrongArgumentCount(Count),
    WrongTypeArgumentCount(Count),
    UnknownField {
        of: Type,
        field: String,
    },
    UnknownReceiver(String),
    InModule {
        module: String,
        name: String,
    },
    Infinite,
    MissingField {
        of: String,
        field: String,
    },
    FieldWrittenTwice {
        of: String,
        field: String,
    },
    NotEquatable(Type),
    DivisorIsZero,
    /// A statement leaves a value behind and nothing takes it.
    Discarded(Type),
    /// A call passes its arguments positionally where the declaration repeats a type.
    Unnamed {
        function: String,
        repeated: Type,
    },
    /// An argument is named something other than the parameter it is passed for.
    Misnamed {
        written: String,
        parameter: String,
    },
    /// A constructor call names its arguments, and a constructor has no names to write.
    NamedConstructor(String),
    /// A call of a prelude function names its arguments, which this module cannot check.
    NamedPrelude(String),
    /// A parameter is a bare `Bool`, so a call of it passes `true` and says no more.
    FlagParameter(String),
}

impl TypeErrorKind {
    /// The code this failure is refused with, which `docs/specs/types.md` lists.
    const fn code(&self) -> Code {
        match self {
            Self::Mismatch { .. } | Self::NotAddable(_) => Code::TypeMismatch,
            Self::WrongArgumentCount(_) | Self::WrongTypeArgumentCount(_) => {
                Code::WrongArgumentCount
            }
            Self::UnknownField { .. } | Self::UnknownReceiver(_) | Self::InModule { .. } => {
                Code::UnknownField
            }
            Self::Infinite => Code::InfiniteType,
            Self::MissingField { .. } => Code::MissingField,
            Self::FieldWrittenTwice { .. } => Code::FieldWrittenTwice,
            Self::NotEquatable(_) => Code::NotEquatable,
            Self::DivisorIsZero => Code::DivisorIsZero,
            Self::Discarded(_) => Code::Discarded,
            Self::Unnamed { .. } => Code::Unnamed,
            Self::Misnamed { .. } => Code::Misnamed,
            Self::NamedConstructor(_) | Self::NamedPrelude(_) => Code::Unnameable,
            Self::FlagParameter(_) => Code::FlagParameter,
        }
    }

    const fn help(&self) -> &'static str {
        match self {
            Self::Mismatch { .. } => "one type is not another, however alike they are held",
            Self::NotAddable(_) => "`+` adds two `Int`s or joins two `String`s",
            Self::WrongArgumentCount(_) => {
                "a call passes one argument for each the declaration lists"
            }
            Self::WrongTypeArgumentCount(_) => {
                "a type is written with one argument for each the declaration lists"
            }
            Self::UnknownField { .. } => "a field is looked up in the record its type declares",
            Self::UnknownReceiver(_) => {
                "declare the type of a parameter or a result so the field can be found"
            }
            Self::InModule { .. } => {
                "a module's names arrive with module loading, which version 0.1 has not"
            }
            Self::Infinite => "one of these two is being used where the other was meant",
            Self::MissingField { .. } => {
                "building a record gives every field a value; update one to change only some"
            }
            Self::FieldWrittenTwice { .. } => "a record gives each of its fields one value",
            Self::NotEquatable(_) => {
                "`==` and `!=` need `Eq`, which version 0.1 gives to `Int`, `Bool`, and `String`"
            }
            Self::DivisorIsZero => "a zero written here is never anything else; drop the division",
            Self::Discarded(_) => "write `_ = ` in front of it to throw the value away on purpose",
            Self::Unnamed { .. } => {
                "a call names its arguments when the declaration gives two parameters one type"
            }
            Self::Misnamed { .. } => {
                "arguments are named in the order the declaration lists its parameters"
            }
            Self::NamedConstructor(_) => {
                "a variant whose values want names declares them as fields and is built as a record"
            }
            Self::NamedPrelude(_) => {
                "only a call of a function this module declares names its arguments"
            }
            Self::FlagParameter(_) => {
                "declare a two-variant type and take that instead, so the call says which of the two"
            }
        }
    }
}

impl fmt::Display for TypeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mismatch { expected, found } => {
                write!(f, "expected `{expected}`, found `{found}`")
            }
            Self::NotAddable(found) => write!(f, "`{found}` cannot be added"),
            Self::WrongArgumentCount(count) => count.fmt_with(f, "argument"),
            Self::WrongTypeArgumentCount(count) => count.fmt_with(f, "type argument"),
            Self::UnknownField { of, field } => write!(f, "`{of}` has no field named `{field}`"),
            Self::UnknownReceiver(field) => {
                write!(
                    f,
                    "the type here is not known, so `{field}` cannot be found"
                )
            }
            Self::InModule { module, name } => {
                write!(
                    f,
                    "`{module}` is a module, and `{name}` cannot be reached inside one yet"
                )
            }
            Self::Infinite => f.write_str("this would have a type that contains itself"),
            Self::MissingField { of, field } => {
                write!(f, "`{of}` needs a field named `{field}`")
            }
            Self::FieldWrittenTwice { of, field } => {
                write!(f, "`{of}` is given `{field}` twice")
            }
            Self::NotEquatable(found) => {
                write!(
                    f,
                    "`{found}` has no `Eq`, so two of them cannot be compared"
                )
            }
            Self::DivisorIsZero => write!(f, "this divisor is zero, so there is no answer"),
            Self::Discarded(left) => write!(f, "`{left}` is left here and nothing takes it"),
            Self::Unnamed { function, repeated } => {
                write!(
                    f,
                    "`{function}` gives two parameters the type `{repeated}`, \
                     so this call names its arguments"
                )
            }
            Self::Misnamed { written, parameter } => {
                write!(
                    f,
                    "this argument is named `{written}`, and the parameter here is `{parameter}`"
                )
            }
            Self::NamedConstructor(called) => {
                write!(
                    f,
                    "`{called}` is a constructor, so it carries its values in order and names none"
                )
            }
            Self::NamedPrelude(called) => {
                write!(
                    f,
                    "`{called}` comes from the prelude, which declares no parameter names to write"
                )
            }
            Self::FlagParameter(function) => {
                write!(
                    f,
                    "this parameter is a `Bool`, so a call of `{function}` passes `true` and says no more"
                )
            }
        }
    }
}

/// What a declaration takes and what a use of it gave, for the two arities that can be wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Count {
    pub(crate) name: String,
    pub(crate) takes: usize,
    pub(crate) given: usize,
}

impl Count {
    fn fmt_with(&self, f: &mut fmt::Formatter<'_>, what: &str) -> fmt::Result {
        let Self { name, takes, given } = self;
        let plural = if *takes == 1 { "" } else { "s" };
        let was_or_were = if *given == 1 { "was" } else { "were" };
        write!(
            f,
            "`{name}` takes {takes} {what}{plural} but {given} {was_or_were} given"
        )
    }
}

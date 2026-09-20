//! The one error type inference can fail with.

use std::fmt;

use lumen_ast::Span;
use lumen_diagnostics::{Code, Diagnostic};
use lumen_resolver::prelude;

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
            Some(self.help()),
        )
    }

    /// The `error:` line, without its prefix.
    #[must_use]
    pub fn message(&self) -> String {
        self.kind.to_string()
    }

    /// The `help:` line, which every one of these has.
    #[must_use]
    pub fn help(&self) -> String {
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
    WrongArgumentCount(Count),
    WrongTypeArgumentCount(Count),
    UnknownField {
        of: Type,
        field: String,
    },
    UnknownReceiver(String),
    Infinite,
    MissingField {
        of: String,
        field: String,
    },
    FieldWrittenTwice {
        of: String,
        field: String,
    },
    /// A whole number written at a type whose instance of `IntegerLiteral` does not hold it.
    LiteralDoesNotFit {
        value: i64,
        at: Type,
        lowest: i64,
        highest: i64,
    },
    /// A bound of an `IntegerLiteral` instance written as something other than one whole number.
    BoundIsNotAWholeNumber {
        bound: String,
        of: String,
    },
    /// An operator written over a type that has no instance of the trait that operator is.
    NoOperator {
        written_as: &'static str,
        of: String,
        at: Type,
    },
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
    NotAPredicate(String),
    /// A name reached inside a module, which the module does not declare.
    NotInModule {
        module: String,
        name: String,
    },
    /// A function reached through a module whose signature names a type that module declares.
    TypeOfAnotherModule {
        module: String,
        name: String,
        /// The type the module keeps to itself, which is the one this one cannot write.
        declared: String,
    },
    /// A generic function reached through a module, which is written where it is declared.
    GenericThroughModule {
        module: String,
        name: String,
    },
    /// A trait method, or a constrained generic, used at a type that has no instance.
    NoInstance {
        of: String,
        at: Type,
    },
    /// A type deriving `Eq` that holds a value of a type that has none.
    HeldTypeHasNoInstance {
        deriving: String,
        held: Type,
        /// Where the value sits in the declaration: a field's name, or a variant's and its own.
        held_as: String,
    },
    /// A parameter of a trait's method that states no type.
    SignatureWithoutType(String),
    /// A declared type holds a value of itself, around a ring that comes back to it.
    HoldsItself {
        /// The types the ring runs through, beginning and ending at the one refused.
        ring: Vec<String>,
    },
}

impl TypeErrorKind {
    /// The code this failure is refused with, which `docs/specs/types.md` lists.
    const fn code(&self) -> Code {
        match self {
            Self::Mismatch { .. } => Code::TypeMismatch,
            Self::WrongArgumentCount(_) | Self::WrongTypeArgumentCount(_) => {
                Code::WrongArgumentCount
            }
            Self::UnknownField { .. } | Self::UnknownReceiver(_) => Code::UnknownField,
            Self::Infinite => Code::InfiniteType,
            Self::MissingField { .. } => Code::MissingField,
            Self::FieldWrittenTwice { .. } => Code::FieldWrittenTwice,
            Self::NoOperator { .. } => Code::NoOperator,
            Self::LiteralDoesNotFit { .. } => Code::LiteralDoesNotFit,
            Self::BoundIsNotAWholeNumber { .. } => Code::BoundIsNotAWholeNumber,
            Self::NoInstance { .. } => Code::NoInstance,
            Self::HeldTypeHasNoInstance { .. } => Code::HeldTypeHasNoInstance,
            Self::SignatureWithoutType(_) => Code::SignatureWithoutType,
            Self::DivisorIsZero => Code::DivisorIsZero,
            Self::Discarded(_) => Code::Discarded,
            Self::Unnamed { .. } => Code::Unnamed,
            Self::Misnamed { .. } => Code::Misnamed,
            Self::NamedConstructor(_) | Self::NamedPrelude(_) => Code::Unnameable,
            Self::FlagParameter(_) => Code::FlagParameter,
            Self::NotAPredicate(_) => Code::NotAPredicate,
            Self::NotInModule { .. } => Code::NotInModule,
            Self::TypeOfAnotherModule { .. } => Code::TypeOfAnotherModule,
            Self::GenericThroughModule { .. } => Code::GenericThroughModule,
            Self::HoldsItself { .. } => Code::HoldsItself,
        }
    }

    /// The one line that says what to do about it, which every one of these has.
    ///
    /// A type with no `Eq` is the one whose answer names the source that gives it one, because
    /// `docs/specs/derive.md` writes that instance for whoever asks and the reader may not know.
    fn help(&self) -> String {
        self.derived_instead()
            .unwrap_or_else(|| self.stated().to_owned())
    }

    /// The derive that would give a type the `Eq` a comparison of it wanted, where that is what
    /// went wrong and `docs/specs/derive.md` writes that instance for whoever asks.
    fn derived_instead(&self) -> Option<String> {
        let Self::NoOperator { of, at, .. } = self else {
            return None;
        };
        if of != prelude::EQ {
            return None;
        }
        let Type::Named { arguments, .. } = at else {
            return None;
        };
        arguments
            .is_empty()
            .then(|| format!("`{at}` gets one by deriving it: write `derive Eq for {at}`"))
    }

    const fn stated(&self) -> &'static str {
        match self {
            Self::Mismatch { .. } => "one type is not another, however alike they are held",
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
            Self::Infinite => "one of these two is being used where the other was meant",
            Self::MissingField { .. } => {
                "building a record gives every field a value; update one to change only some"
            }
            Self::FieldWrittenTwice { .. } => "a record gives each of its fields one value",
            Self::NoOperator { .. } => {
                "an operator is a trait method, so a type gets one by writing that trait's instance"
            }
            Self::LiteralDoesNotFit { .. } => {
                "write a whole number the type holds, or widen what its instance says it holds"
            }
            Self::BoundIsNotAWholeNumber { .. } => {
                "a bound is read rather than run, so write it as one whole number and nothing else"
            }
            Self::NoInstance { .. } => {
                "write the instance, or constrain the type parameter the call is made at"
            }
            Self::HeldTypeHasNoInstance { .. } => {
                "a derived `Eq` compares by the `Eq` of what it holds; give the held type one"
            }
            Self::SignatureWithoutType(_) => {
                "a signature has no body to read a type off, so it writes each one out"
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
            Self::NotAPredicate(_) => "begin the name with `is_`, `has_`, `can_`, or `should_`",
            Self::NotInModule { .. } => {
                "a module declares the functions it offers, and nothing else is a name it has"
            }
            Self::TypeOfAnotherModule { .. } => {
                "a signature written in types both modules have is what one module offers another"
            }
            Self::GenericThroughModule { .. } => {
                "write it in the module that reaches it, or give it a signature at one set of types"
            }
            Self::HoldsItself { .. } => {
                "hold an `Option` of it, which is how a type holds another of its own kind"
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
            Self::WrongArgumentCount(count) => count.fmt_with(f, "argument"),
            Self::WrongTypeArgumentCount(count) => count.fmt_with(f, "type argument"),
            Self::UnknownField { of, field } => write!(f, "`{of}` has no field named `{field}`"),
            Self::UnknownReceiver(field) => {
                write!(
                    f,
                    "the type here is not known, so `{field}` cannot be found"
                )
            }
            Self::NotInModule { .. }
            | Self::TypeOfAnotherModule { .. }
            | Self::GenericThroughModule { .. } => f.write_str(&self.how_a_module_is_reached()),
            Self::HoldsItself { ring } => {
                let (first, rest) = ring.split_first().expect("a ring runs through one type");
                write!(f, "`{first}` holds ")?;
                for held in rest {
                    write!(f, "`{held}`, which holds ")?;
                }
                write!(f, "`{first}`")
            }
            Self::Infinite => f.write_str("this would have a type that contains itself"),
            Self::MissingField { of, field } => {
                write!(f, "`{of}` needs a field named `{field}`")
            }
            Self::FieldWrittenTwice { of, field } => {
                write!(f, "`{of}` is given `{field}` twice")
            }
            Self::LiteralDoesNotFit { .. } | Self::BoundIsNotAWholeNumber { .. } => {
                f.write_str(&self.how_a_whole_number_is_held())
            }
            Self::NoOperator { written_as, of, at } => {
                write!(
                    f,
                    "`{at}` has no `{of}`, so `{written_as}` is not written over it"
                )
            }
            Self::NoInstance { of, at } => {
                write!(f, "`{at}` has no instance of `{of}`")
            }
            Self::HeldTypeHasNoInstance { .. } => f.write_str(&self.what_it_holds()),
            Self::SignatureWithoutType(name) => {
                write!(
                    f,
                    "`{name}` states no type, and a signature is never inferred"
                )
            }
            Self::DivisorIsZero => write!(f, "this divisor is zero, so there is no answer"),
            Self::Discarded(left) => write!(f, "`{left}` is left here and nothing takes it"),
            Self::Unnamed { .. }
            | Self::Misnamed { .. }
            | Self::NamedConstructor(_)
            | Self::NamedPrelude(_)
            | Self::FlagParameter(_)
            | Self::NotAPredicate(_) => f.write_str(&self.how_it_is_written()),
        }
    }
}

impl TypeErrorKind {
    /// The message of a refusal about how a call or a declaration is written.
    ///
    /// `docs/specs/arguments.md` and `docs/specs/naming.md` state these, and they read as one
    /// group: each says what the author wrote rather than what type met what. Every other kind is
    /// about a type, and [`fmt::Display`] writes those itself.
    ///
    /// A kind reaches here only from the arm of [`fmt::Display`] that names it, so a new one is
    /// added to both or to neither: `Display` matches every variant, and leaving one out of that
    /// match is a compile error rather than a message nothing writes.
    /// The message of a refusal about a name reached through a module.
    ///
    /// `docs/specs/modules.md` states these, and they read as one group: each says what the
    /// module on the far side of the dot does or does not offer, rather than what type met what.
    ///
    /// A kind reaches here only from the arm of [`fmt::Display`] that names it, on the terms
    /// [`Self::how_it_is_written`] states.
    fn how_a_module_is_reached(&self) -> String {
        match self {
            Self::NotInModule { module, name } => format!("`{module}` declares no `{name}`"),
            Self::TypeOfAnotherModule {
                module,
                name,
                declared,
            } => {
                format!("`{module}.{name}` names `{declared}`, which `{module}` keeps to itself")
            }
            Self::GenericThroughModule { module, name } => {
                format!("`{module}.{name}` is generic, so `{module}` alone writes it")
            }
            _ => unreachable!("a kind reaches here only from the arm of `Display` that names it"),
        }
    }

    fn how_it_is_written(&self) -> String {
        match self {
            Self::Unnamed { function, repeated } => {
                format!(
                    "`{function}` gives two parameters the type `{repeated}`, \
                     so this call names its arguments"
                )
            }
            Self::Misnamed { written, parameter } => {
                format!(
                    "this argument is named `{written}`, and the parameter here is `{parameter}`"
                )
            }
            Self::NamedConstructor(called) => {
                format!(
                    "`{called}` is a constructor, so it carries its values in order and names none"
                )
            }
            Self::NamedPrelude(called) => {
                format!(
                    "`{called}` comes from the prelude, which declares no parameter names to write"
                )
            }
            Self::FlagParameter(function) => {
                format!(
                    "this parameter is a `Bool`, so a call of `{function}` passes `true` and says no more"
                )
            }
            Self::NotAPredicate(function) => {
                format!(
                    "`{function}` gives back a `Bool`, so its name asks the question it answers"
                )
            }
            _ => unreachable!("a kind reaches here only from the arm of `Display` that names it"),
        }
    }

    /// The two refusals about a whole number, which `docs/specs/literals.md` states.
    /// What a type deriving `Eq` holds that has none, which is what the derive is refused for.
    fn what_it_holds(&self) -> String {
        let Self::HeldTypeHasNoInstance {
            deriving,
            held,
            held_as,
        } = self
        else {
            unreachable!("a kind reaches here only from the arm of `Display` that names it")
        };
        format!("`{deriving}` derives `Eq`, and the `{held}` it holds as `{held_as}` has none")
    }

    fn how_a_whole_number_is_held(&self) -> String {
        match self {
            Self::LiteralDoesNotFit {
                value,
                at,
                lowest,
                highest,
            } => format!("`{value}` does not fit `{at}`, which holds `{lowest}` to `{highest}`"),
            Self::BoundIsNotAWholeNumber { bound, of } => {
                format!("`{bound}` of `{of}` is read rather than run, so it is one whole number")
            }
            _ => unreachable!("a kind reaches here only from the arm of `Display` that names it"),
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

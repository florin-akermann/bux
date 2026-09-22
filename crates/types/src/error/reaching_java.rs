//! What a refusal about an `extern` declaration says, and the rule it says it against.
//!
//! `docs/specs/interop.md` states these, and they read as one group: each says that the boundary
//! is a signature and the Java name beside it and nothing more. The group is a type of its own,
//! so a new kind of it is added here alone, and every one of them is stated here alone.

use std::fmt;

use lumen_diagnostics::Code;

use crate::boundary::Crossing;
use crate::types::Type;

/// What an `extern` declaration does not reach, and why the boundary does not carry it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ReachingJava {
    /// An `extern` signature names a type no Java member takes or gives back.
    DoesNotCross {
        written: Type,
        /// Where in the signature the type sits, which is what decides whether it may be there.
        crossing: Crossing,
    },
    /// An `extern` states something that is no Java name, in the place it states one.
    NotAJavaName(String),
    /// A derive names a type an `extern type` declares.
    DerivesAForeignType(String),
    /// An `extern` whose kind reaches a class is written with a signature naming none.
    ReachesNoClass(String),
    /// An `extern new` builds what it gives back, and an `extern type` named that an interface.
    BuildsAnInterface(Type),
    /// An `extern` says its member gives a width, and what it gives back is no `Int`.
    WidensNoInt {
        /// The word the declaration wrote, which is `int` or `char`.
        width: String,
        written: Type,
    },
    /// An `extern` narrows a parameter to an `int`, and what that parameter takes is no `Int`.
    NarrowsNoInt(Type),
    /// An `extern` narrows a parameter, and what it gives back is no `Option` to say `None` with.
    NarrowsWithoutOption(Type),
}

impl ReachingJava {
    /// The code this failure is refused with, which `docs/specs/diagnostics.md` lists.
    pub(super) const fn code(&self) -> Code {
        match self {
            Self::DoesNotCross { .. } => Code::DoesNotCross,
            Self::NotAJavaName(_) => Code::NotAJavaName,
            Self::DerivesAForeignType(_) => Code::DerivesAForeignType,
            Self::ReachesNoClass(_) => Code::ReachesNoClass,
            Self::BuildsAnInterface(_) => Code::BuildsAnInterface,
            Self::WidensNoInt { .. } | Self::NarrowsNoInt(_) => Code::WidensNoInt,
            Self::NarrowsWithoutOption(_) => Code::NarrowsWithoutOption,
        }
    }

    /// The rule this refusal is said against.
    pub(super) const fn the_rule(&self) -> &'static str {
        match self {
            Self::DoesNotCross { .. } => {
                "a boundary carries `Bool`, `Int`, `String`, a `List`, and a type an `extern` names"
            }
            Self::NotAJavaName(_) => {
                "a Java name is its segments, each a name, with a dot between two of them"
            }
            Self::DerivesAForeignType(_) => {
                "write an `instance` over `extern` declarations instead"
            }
            Self::ReachesNoClass(_) => "a class is `String`, or a type an `extern type` declares",
            Self::BuildsAnInterface(_) => {
                "a `new` builds a class; reach one through a member of a class instead"
            }
            Self::WidensNoInt { .. } => {
                "a width widens to an `Int`; drop the word, or give an `Int` back"
            }
            Self::NarrowsNoInt(_) => "an `int` narrows an `Int`; drop the word, or take an `Int`",
            Self::NarrowsWithoutOption(_) => {
                "give back an `Option`, which is `None` where an argument does not fit"
            }
        }
    }
}

impl fmt::Display for ReachingJava {
    /// What the declaration could not reach, in the words the reader is shown.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DoesNotCross { written, crossing } => {
                let place = crossing.what_a_member_does();
                write!(f, "`{written}` is no type a Java member {place}")
            }
            Self::NotAJavaName(written) => write!(f, "`{written}` is no Java name"),
            Self::DerivesAForeignType(named) => {
                write!(
                    f,
                    "`{named}` is an extern type, and a derive reads what a type holds"
                )
            }
            Self::ReachesNoClass(reaches) => {
                write!(
                    f,
                    "a `{reaches}` reaches a class, and this signature names none"
                )
            }
            Self::BuildsAnInterface(built) => {
                write!(f, "a `new` builds a class, and `{built}` is an interface")
            }
            Self::WidensNoInt { width, written } => {
                write!(
                    f,
                    "`{width}` widens to an `Int`, and this signature gives back `{written}`"
                )
            }
            Self::NarrowsNoInt(written) => {
                write!(
                    f,
                    "`int` narrows an `Int`, and this parameter takes `{written}`"
                )
            }
            Self::NarrowsWithoutOption(written) => {
                write!(
                    f,
                    "an `int` parameter narrows an `Int`, and this signature gives back `{written}`"
                )
            }
        }
    }
}

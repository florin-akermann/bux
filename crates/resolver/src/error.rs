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
}

impl ResolveErrorKind {
    /// The failure a name missing from `namespace` amounts to.
    pub(crate) fn missing(namespace: Namespace, text: &str) -> Self {
        match namespace {
            Namespace::Value => Self::NoValue(text.to_owned()),
            Namespace::Type => Self::NoType(text.to_owned()),
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
        }
    }

    const fn help(&self) -> &'static str {
        match self {
            Self::NoValue(_) | Self::NoType(_) => {
                "a name is declared in this file, imported, or supplied by the prelude"
            }
            Self::DeclaredTwice(_) => "one name has one definition; rename one of the two",
            Self::Shadowed(_) => "rename the inner one; Lumen never hides a name",
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
        }
    }
}

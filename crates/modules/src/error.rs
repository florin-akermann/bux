//! The one refusal loading can fail with, said about the file whose import it points at.

use std::fmt;

use lumen_ast::Span;
use lumen_diagnostics::{Code, Diagnostic};

use crate::package::Keyword;

/// Why loading failed, and which import it was following when it did.
///
/// Loading stops at the first refusal, so there is exactly one of these per failed run. The
/// wording is specified in `docs/specs/modules.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadError {
    kind: LoadErrorKind,
    span: Span,
}

impl LoadError {
    pub(crate) const fn at(span: Span, kind: LoadErrorKind) -> Self {
        Self { kind, span }
    }

    /// The source the refusal points at, which lies in the file that wrote the import.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// This refusal as the diagnostic the reader is shown.
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
pub(crate) enum LoadErrorKind {
    /// An import names a module neither a file beside it nor a package it reaches holds.
    NoSuchModule(String),
    /// A ring of imports, named in the order it runs, beginning and ending at the same module.
    Ring(Vec<String>),
    /// A manifest line is not the line the keyword it carries says belongs there.
    LineIsNot(Keyword),
    /// A manifest line states its keyword and then something other than one word.
    NotOneWord(Keyword),
    /// A `depends` names a directory that holds no manifest, so there is no package there.
    NoSuchPackage(String),
    /// A manifest states two `depends` naming one and the same directory.
    DependsTwice(String),
    /// An import names a module that two files, reached along two routes, both claim.
    InTwoFiles {
        module: String,
        first: String,
        second: String,
    },
}

impl LoadErrorKind {
    /// The code this refusal is refused with, which `docs/specs/modules.md` lists.
    const fn code(&self) -> Code {
        match self {
            Self::NoSuchModule(_) => Code::NoSuchModule,
            Self::Ring(_) => Code::RingOfImports,
            Self::LineIsNot(_) | Self::NotOneWord(_) | Self::DependsTwice(_) => Code::NotAManifest,
            Self::NoSuchPackage(_) => Code::NoSuchPackage,
            Self::InTwoFiles { .. } => Code::ModuleIsTwoFiles,
        }
    }

    const fn help(&self) -> &'static str {
        match self {
            Self::NoSuchModule(_) => {
                "a module is a file beside this one, or one of a package it depends on"
            }
            Self::Ring(_) => {
                "a module is compiled after what it imports, and a ring has no such order"
            }
            Self::LineIsNot(_) => {
                "a manifest is `package`, then `version`, then a `depends` for each dependency"
            }
            Self::NotOneWord(_) => "a manifest line is a keyword, one space, and one word",
            Self::DependsTwice(_) => "a manifest states one `depends` for each package it reaches",
            Self::NoSuchPackage(_) => "a package is a directory holding `bux.package`",
            Self::InTwoFiles { .. } => "one name has one definition; rename one of the two modules",
        }
    }
}

impl fmt::Display for LoadErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchModule(name) => write!(f, "there is no module named `{name}`"),
            Self::Ring(ring) => {
                let (first, rest) = ring.split_first().expect("a ring runs through one module");
                write!(f, "`{first}` imports ")?;
                for imported in rest {
                    write!(f, "`{imported}`, which imports ")?;
                }
                write!(f, "`{first}`")
            }
            Self::LineIsNot(keyword) => write!(f, "`{keyword}` is what a manifest states here"),
            Self::NotOneWord(keyword) => {
                write!(f, "`{keyword}` states one word, and this line does not")
            }
            Self::DependsTwice(named) => write!(f, "`{named}` is depended on twice"),
            Self::NoSuchPackage(named) => write!(f, "there is no package in `{named}`"),
            Self::InTwoFiles {
                module,
                first,
                second,
            } => write!(f, "`{module}` is both `{first}` and `{second}`"),
        }
    }
}

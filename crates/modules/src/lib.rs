//! Loading: the file a command names, and every module it reaches, read and parsed.
//!
//! The phase runs before name resolution and yields a [`Loaded`], holding one [`Module`] per
//! file, dependencies before dependents. Nothing is merged: each module keeps its own source and
//! its own tree, and is compiled as the module it is. `docs/specs/modules.md` is the
//! specification, and `docs/design.md` section 16 is the rule it enforces: one file is one
//! module, named by its file. `docs/specs/packages.md` states the one place other than beside
//! the importing file that an import is answered from.

mod error;
mod load;
mod package;

use std::io;
use std::path::{Path, PathBuf};

use lumen_ast::Program;
use lumen_diagnostics::Diagnostic;

pub use crate::error::LoadError;
pub use crate::load::{load, written};
pub use crate::package::{MANIFEST, SUFFIX};

/// Every module a program reaches, in the order they are compiled.
///
/// A module sits below every module it imports, so each one is compiled with the surfaces of
/// everything it reaches already worked out.
#[derive(Clone, Debug)]
pub struct Loaded {
    modules: Vec<Module>,
}

impl Loaded {
    /// Every module, dependencies before dependents, ending with the one the command named.
    #[must_use]
    pub fn modules(&self) -> &[Module] {
        &self.modules
    }

    /// The same modules, in the same order, to be compiled one after another.
    #[must_use]
    pub fn into_modules(self) -> Vec<Module> {
        self.modules
    }
}

/// One file, read and parsed, under the name the file gives it.
#[derive(Clone, Debug)]
pub struct Module {
    name: String,
    path: PathBuf,
    source: String,
    program: Program,
}

impl Module {
    /// The name this module is imported and reached under, which is its file's.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The file it was read from, which is what a refusal about it names.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The text it was read from, which is what a refusal about it points into.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The tree the grammar made of it.
    #[must_use]
    pub const fn program(&self) -> &Program {
        &self.program
    }
}

/// Why a program could not be loaded, which is either a file or a refusal of one.
#[derive(Debug)]
pub enum NotLoaded {
    /// A file loading was told to read, and the reason it could not be read.
    Unreadable { path: PathBuf, why: io::Error },
    /// A refusal of one of the files, said about that file rather than the one named.
    Refused(Box<Refusal>),
}

impl NotLoaded {
    /// `diagnostic`, carrying the file it points into so a reader is shown the right source.
    pub(crate) fn refused(diagnostic: &Diagnostic, path: &Path, source: &str) -> Self {
        Self::Refused(Box::new(Refusal {
            path: path.to_path_buf(),
            source: source.to_owned(),
            diagnostic: diagnostic.clone(),
        }))
    }
}

/// One file the compiler will not have, and the text the refusal is read against.
#[derive(Clone, Debug)]
pub struct Refusal {
    path: PathBuf,
    source: String,
    diagnostic: Diagnostic,
}

impl Refusal {
    /// The file it is shown against, which is the file that wrote what is refused.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The text it points into.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// What the reader is shown.
    #[must_use]
    pub const fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

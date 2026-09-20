//! Every phase the front end has, run over each module a program reaches.
//!
//! Loading says which modules those are and which order they compile in, and each one is checked
//! with the surfaces of everything it imports already worked out. `docs/specs/modules.md` states
//! the order and why a ring of imports has none.

use std::path::{Path, PathBuf};

use lumen_diagnostics::Diagnostic;
use lumen_exhaustiveness::check as exhaustive;
use lumen_modules::{NotLoaded, load};
use lumen_types::{Imported, TypedProgram, check as check_types};

/// Every module a program reaches, checked, dependencies before dependents.
pub(crate) struct Checked {
    modules: Vec<Module>,
    imported: Imported,
}

impl Checked {
    /// What every module loaded offers, which is what typing one more of them reads.
    pub(crate) const fn imported(&self) -> &Imported {
        &self.imported
    }

    /// Every module, in the order they are compiled, ending with the one the command named.
    pub(crate) fn modules(&self) -> &[Module] {
        &self.modules
    }

    /// The module the command named, which is the one a program is run through.
    pub(crate) fn root(&self) -> &Module {
        self.modules
            .last()
            .expect("loading reads the file it is given")
    }
}

/// One module, and the types inference gave it.
pub(crate) struct Module {
    loaded: lumen_modules::Module,
    typed: TypedProgram,
}

impl Module {
    /// The name it is reached under, which is the name of its file.
    pub(crate) fn name(&self) -> &str {
        self.loaded.name()
    }

    /// The file it was read from, which is what a refusal about it names.
    pub(crate) fn path(&self) -> &Path {
        self.loaded.path()
    }

    /// The text it was read from, which is what a refusal about it points into.
    pub(crate) fn source(&self) -> &str {
        self.loaded.source()
    }

    /// The types inference gave it.
    pub(crate) const fn typed(&self) -> &TypedProgram {
        &self.typed
    }
}

/// Checks the module at `path` and every module it reaches, each after what it imports.
pub(crate) fn checked(path: &Path) -> Result<Checked, NotCompiled> {
    let loaded = load(path).map_err(NotCompiled::of)?;
    let mut imported = Imported::default();
    let mut modules = Vec::new();
    for loaded in loaded.into_modules() {
        let typed = match accepted(loaded.source(), &imported) {
            Ok(typed) => typed,
            Err(diagnostic) => {
                return Err(NotCompiled::refused(
                    vec![diagnostic],
                    loaded.path(),
                    loaded.source(),
                ));
            }
        };
        imported = imported.offering(loaded.name(), typed.surface().clone());
        modules.push(Module { loaded, typed });
    }
    Ok(Checked { modules, imported })
}

/// Every phase the front end has, run in order over one module, stopping at the first refusal.
pub(crate) fn accepted(source: &str, imported: &Imported) -> Result<TypedProgram, Diagnostic> {
    lumen_format::check(source).map_err(|error| error.diagnostic())?;
    typed(source, imported)
}

/// Every phase after canonical form, which is all of them a module the compiler wrote needs.
///
/// Canonical form is a rule about what an author writes, and `docs/specs/formatting.md` leaves
/// the text of a comment alone. An example is a comment, so holding a module written around one
/// to canonical form would hold the author to a form nothing spells out and `lumen fmt` cannot
/// repair.
pub(crate) fn typed(source: &str, imported: &Imported) -> Result<TypedProgram, Diagnostic> {
    let program = lumen_parser::parse(source).map_err(|error| error.diagnostic())?;
    let resolved = lumen_resolver::resolve(program).map_err(|error| error.diagnostic())?;
    let inferred = check_types(resolved, imported).map_err(|error| error.diagnostic())?;
    exhaustive(&inferred).map_err(|error| error.diagnostic())?;
    Ok(inferred)
}

/// Why a program was not compiled, which is either a file or a refusal of one.
pub(crate) enum NotCompiled {
    /// A file could not be read, and the reason has already been said.
    Unusable,
    /// A refusal, said about the file it points into rather than the file the command named.
    Refused(Refusal),
}

impl NotCompiled {
    /// What loading failing with `error` amounts to, having said what cannot be read.
    fn of(error: NotLoaded) -> Self {
        match error {
            NotLoaded::Unreadable { path, why } => {
                eprintln!("error: {}: {why}", path.display());
                Self::Unusable
            }
            NotLoaded::Refused(refused) => Self::Refused(Refusal {
                diagnostics: vec![refused.diagnostic().clone()],
                source: refused.source().to_owned(),
                path: refused.path().to_path_buf(),
            }),
        }
    }

    /// Everything wrong with one module, shown against the file that module was read from.
    pub(crate) fn refused(diagnostics: Vec<Diagnostic>, path: &Path, source: &str) -> Self {
        Self::Refused(Refusal {
            diagnostics,
            source: source.to_owned(),
            path: path.to_path_buf(),
        })
    }
}

/// What the compiler will not have, said about the file it points into.
pub(crate) struct Refusal {
    diagnostics: Vec<Diagnostic>,
    source: String,
    path: PathBuf,
}

impl Refusal {
    /// Everything wrong with the file, in the order a reader is shown them.
    pub(crate) fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// The text the refusals point into.
    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    /// The file the refusals are shown against.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

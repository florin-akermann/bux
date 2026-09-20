//! The walk: the file a command names, every module it reaches, and the order they compile in.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use lumen_ast::{Import, Item, Program};
use lumen_resolver::library;

use crate::error::{LoadError, LoadErrorKind};
use crate::{Loaded, Module, NotLoaded};

/// What a module is written in, which is how an import finds the file it names.
const SUFFIX: &str = "lm";

/// Reads the module at `path` and every module it reaches, dependencies before dependents.
///
/// `docs/specs/modules.md` states where an import finds its file and why a ring has no order.
///
/// # Errors
///
/// Returns the first file that cannot be read, the first import that names no file beside the
/// one that wrote it, the first ring of imports, and the first file the grammar refuses.
pub fn load(path: &Path) -> Result<Loaded, NotLoaded> {
    let mut loader = Loader::default();
    loader.reach(&named(path), path)?;
    Ok(Loaded {
        modules: loader.modules,
    })
}

/// The module a file declares, which is the name of the file it is written in.
fn named(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Every module read so far, and the chain of imports being followed right now.
#[derive(Default)]
struct Loader {
    modules: Vec<Module>,
    read: HashSet<String>,
    following: Vec<String>,
}

impl Loader {
    /// Reads `name` out of the file at `path`, and everything it imports.
    fn reach(&mut self, name: &str, path: &Path) -> Result<(), NotLoaded> {
        let source = std::fs::read_to_string(path).map_err(|why| NotLoaded::Unreadable {
            path: path.to_path_buf(),
            why,
        })?;
        self.walk(name, path, source)
    }

    /// Puts `name` below every module it reaches, having read the source it is written in.
    fn walk(&mut self, name: &str, path: &Path, source: String) -> Result<(), NotLoaded> {
        let program = lumen_parser::parse(&source)
            .map_err(|error| NotLoaded::refused(&error.diagnostic(), path, &source))?;
        self.following.push(name.to_owned());
        for import in imports(&program) {
            self.follow(import, path, &source)?;
        }
        self.following.pop();
        self.read.insert(name.to_owned());
        self.modules.push(Module {
            name: name.to_owned(),
            path: path.to_path_buf(),
            source,
            program,
        });
        Ok(())
    }

    /// Reads what `import` names, unless the compiler supplies it or it has been read already.
    ///
    /// A module of the library is read out of the source the compiler carries rather than out of
    /// a file, which `docs/specs/library.md` states: the name is the library's, and a file of it
    /// beside the importing one does not shadow it. That is settled before a ring is looked for,
    /// because a file of that name is not what the import reached and so is no part of a ring.
    /// The prelude's name is the library's too, and nothing imports it, so an import of it names
    /// no module a program may reach.
    fn follow(&mut self, import: &Import, from: &Path, source: &str) -> Result<(), NotLoaded> {
        let name = import.module.text.as_str();
        if lumen_types::supplies(name) || self.read.contains(name) {
            return Ok(());
        }
        if library::source_of(name).is_some() {
            let Some(carried) = library::imported(name) else {
                return Err(missing(import, from, source));
            };
            return self.walk(name, &written_in(name), carried.to_owned());
        }
        if let Some(opened) = self.following.iter().position(|held| held == name) {
            let ring = LoadErrorKind::Ring(self.following[opened..].to_vec());
            return Err(refusal(
                &LoadError::at(import.module.span, ring),
                from,
                source,
            ));
        }
        let beside = beside(from, name);
        if !beside.is_file() {
            return Err(missing(import, from, source));
        }
        self.reach(name, &beside)
    }
}

/// An import that reaches nothing, said about the file that wrote it.
fn missing(import: &Import, from: &Path, source: &str) -> NotLoaded {
    let missing = LoadErrorKind::NoSuchModule(import.module.text.clone());
    refusal(&LoadError::at(import.module.span, missing), from, source)
}

/// The file a library module is written in, which is where the compiler's own source holds it.
fn written_in(name: &str) -> PathBuf {
    Path::new("library").join(name).with_extension(SUFFIX)
}

/// The file `name` is written in, which sits beside the file that imports it.
fn beside(from: &Path, name: &str) -> PathBuf {
    from.parent()
        .unwrap_or_else(|| Path::new("."))
        .join(name)
        .with_extension(SUFFIX)
}

/// Every module `program` imports, in the order it writes them.
fn imports(program: &Program) -> impl Iterator<Item = &Import> {
    program.items.iter().filter_map(|item| match item {
        Item::Import(import) => Some(import),
        Item::Type(_)
        | Item::Trait(_)
        | Item::Instance(_)
        | Item::Derive(_)
        | Item::Function(_) => None,
    })
}

/// A refusal of loading, said about the file whose import it points at.
fn refusal(error: &LoadError, path: &Path, source: &str) -> NotLoaded {
    NotLoaded::refused(&error.diagnostic(), path, source)
}

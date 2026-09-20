//! The walk: the file a command names, every module it reaches, and the order they compile in.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lumen_ast::{Import, Item, Program};
use lumen_resolver::library;

use crate::error::{LoadError, LoadErrorKind};
use crate::package::{self, Route, SUFFIX};
use crate::{Loaded, Module, NotLoaded};

/// Reads the module at `path` and every module it reaches, dependencies before dependents.
///
/// `docs/specs/modules.md` states where an import finds its file, and `docs/specs/packages.md`
/// states the one other place it looks and why a ring has no order.
///
/// # Errors
///
/// Returns the first file that cannot be read, the first import that names no module anything
/// reaches, the first ring of imports, the first manifest that is not one, the first name two
/// files both claim, and the first file the grammar refuses.
pub fn load(path: &Path) -> Result<Loaded, NotLoaded> {
    let mut loader = Loader::default();
    loader.reach(&named(path), Route::to(path.to_path_buf()))?;
    Ok(Loaded {
        modules: loader.modules,
    })
}

/// Every module `written` reaches, as though it were the file at `path`.
///
/// `lumen test` writes a module out of the module under test, which `docs/specs/doc-examples.md`
/// states, and that module imports what the one under test imports and `io` besides. It stands
/// where that file stands, so its imports are answered exactly where the file's are. Nothing is
/// read from `path`: it is where the imports are looked for, and what a refusal about it names.
///
/// # Errors
///
/// The same as [`load`], for the modules this one imports.
pub fn written(written: &str, path: &Path) -> Result<Loaded, NotLoaded> {
    let mut loader = Loader::default();
    let depends = package::reachable_from(directory_of(path))?;
    let whence = Whence {
        at: Route::to(path.to_path_buf()),
        depends,
    };
    loader.walk(&named(path), written.to_owned(), &whence)?;
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

/// Every module read so far, by the file it was read from, and the imports being followed now.
#[derive(Default)]
struct Loader {
    modules: Vec<Module>,
    read: HashMap<String, Route>,
    following: Vec<String>,
}

impl Loader {
    /// Reads `name` out of the file `route` reaches, and everything it imports.
    fn reach(&mut self, name: &str, route: Route) -> Result<(), NotLoaded> {
        let source =
            std::fs::read_to_string(route.shown()).map_err(|why| NotLoaded::Unreadable {
                path: route.shown().to_path_buf(),
                why,
            })?;
        let depends = package::reachable_from(directory_of(route.shown()))?;
        self.walk(name, source, &Whence { at: route, depends })
    }

    /// Reads `name` out of the source the compiler carries, which sits in no package at all.
    fn carried(&mut self, name: &str, source: String) -> Result<(), NotLoaded> {
        let at = Route::to(written_in(name));
        self.walk(
            name,
            source,
            &Whence {
                at,
                depends: Vec::new(),
            },
        )
    }

    /// Puts `name` below every module it reaches, having read the source it is written in.
    fn walk(&mut self, name: &str, source: String, whence: &Whence) -> Result<(), NotLoaded> {
        let program = lumen_parser::parse(&source)
            .map_err(|error| NotLoaded::refused(&error.diagnostic(), whence.path(), &source))?;
        self.following.push(name.to_owned());
        for import in imports(&program) {
            self.follow(import, whence, &source)?;
        }
        self.following.pop();
        self.read.insert(name.to_owned(), whence.at.clone());
        self.modules.push(Module {
            name: name.to_owned(),
            path: whence.path().to_path_buf(),
            source,
            program,
        });
        Ok(())
    }

    /// Reads what `import` names, unless the compiler supplies it or it has been read already.
    ///
    /// A module of the library is read out of the source the compiler carries rather than out of
    /// a file, which `docs/specs/library.md` states: the name is the library's, and a file of it
    /// beside the importing one does not shadow it. That is settled before a file is looked for,
    /// because a file of that name is not what the import reached. The prelude's name is the
    /// library's too, and nothing imports it, so an import of it names no module a program may
    /// reach.
    ///
    /// Everything left is a file, and which file it is settles before anything else does. A
    /// module already read under that name is that module only where it is that same file: one
    /// name has one definition, and two files claiming a name are refused rather than picked
    /// between, whichever of them was read first.
    fn follow(&mut self, import: &Import, whence: &Whence, source: &str) -> Result<(), NotLoaded> {
        let name = import.module.text.as_str();
        if library::source_of(name).is_some() {
            return self.library(import, whence, source);
        }
        let found = file_of(import, whence, source)?;
        if let Some(opened) = self.following.iter().position(|held| held == name) {
            let ring = LoadErrorKind::Ring(self.following[opened..].to_vec());
            return Err(whence.refusing(&LoadError::at(import.module.span, ring), source));
        }
        if let Some(read) = self.read.get(name) {
            if read.is_the_same_as(&found) {
                return Ok(());
            }
            let both = in_two_files(name, read, &found);
            return Err(whence.refusing(&LoadError::at(import.module.span, both), source));
        }
        self.reach(name, found)
    }

    /// Reads what `import` names out of the source the compiler carries it in.
    fn library(&mut self, import: &Import, whence: &Whence, source: &str) -> Result<(), NotLoaded> {
        let name = import.module.text.as_str();
        if self.read.contains_key(name) {
            return Ok(());
        }
        let Some(carried) = library::imported(name) else {
            return Err(missing(import, whence, source));
        };
        self.carried(name, carried.to_owned())
    }
}

/// Where one module's imports are looked for: beside the file, and in what its package depends on.
struct Whence {
    at: Route,
    depends: Vec<Route>,
}

impl Whence {
    /// The file `name` would be written in beside this module, whether or not it is written.
    fn beside(&self, name: &str) -> PathBuf {
        module_in(directory_of(self.path()), name)
    }

    /// `error` as a refusal of this module, shown against the file that wrote the import.
    fn refusing(&self, error: &LoadError, source: &str) -> NotLoaded {
        NotLoaded::refused(&error.diagnostic(), self.path(), source)
    }

    /// The file this module is written in, which is what a refusal about it names.
    fn path(&self) -> &Path {
        self.at.shown()
    }
}

/// The file `import` names: the one beside the importing module, else a dependency's.
///
/// Beside wins, so a package's own modules win over the ones its dependencies hold. Two
/// dependencies holding the name is no order at all, and is refused as two files are.
fn file_of(import: &Import, whence: &Whence, source: &str) -> Result<Route, NotLoaded> {
    let name = import.module.text.as_str();
    let beside = whence.beside(name);
    if beside.is_file() {
        return Ok(Route::to(beside));
    }
    match holding(&whence.depends, name).as_slice() {
        [] => Err(missing(import, whence, source)),
        [one] => Ok(Route::to(one.clone())),
        [first, second, ..] => {
            let both = in_two_files(name, &Route::to(first.clone()), &Route::to(second.clone()));
            Err(whence.refusing(&LoadError::at(import.module.span, both), source))
        }
    }
}

/// The file each package of `depends` holds a module called `name` in, where it holds one.
fn holding(depends: &[Route], name: &str) -> Vec<PathBuf> {
    depends
        .iter()
        .map(|package| module_in(package.shown(), name))
        .filter(|file| file.is_file())
        .collect()
}

/// One name claimed by two files, each named by the route the imports reached it along.
fn in_two_files(module: &str, first: &Route, second: &Route) -> LoadErrorKind {
    LoadErrorKind::InTwoFiles {
        module: module.to_owned(),
        first: first.shown().display().to_string(),
        second: second.shown().display().to_string(),
    }
}

/// An import that reaches nothing, said about the file that wrote it.
fn missing(import: &Import, whence: &Whence, source: &str) -> NotLoaded {
    let missing = LoadErrorKind::NoSuchModule(import.module.text.clone());
    whence.refusing(&LoadError::at(import.module.span, missing), source)
}

/// The file a library module is written in, which is where the compiler's own source holds it.
fn written_in(name: &str) -> PathBuf {
    module_in(Path::new("library"), name)
}

/// The file a module called `name` is written in, in `directory`, whether or not it is written.
fn module_in(directory: &Path, name: &str) -> PathBuf {
    directory.join(name).with_extension(SUFFIX)
}
/// The directory a file sits in, which is where a module beside it is written.
fn directory_of(path: &Path) -> &Path {
    path.parent().unwrap_or(Path::new("."))
}

/// Every module `program` imports, in the order it writes them.
fn imports(program: &Program) -> impl Iterator<Item = &Import> {
    program.items.iter().filter_map(|item| match item {
        Item::Import(import) => Some(import),
        Item::Type(_)
        | Item::Trait(_)
        | Item::Instance(_)
        | Item::Derive(_)
        | Item::Extern(_)
        | Item::Function(_) => None,
    })
}

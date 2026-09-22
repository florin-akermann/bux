//! The library the compiler carries, which is Lumen source rather than anything written in Rust.
//!
//! `docs/specs/library.md` says where the source is and how it is found: each module is read into
//! the binary at build time, so a compiler that runs at all has its library. There is no install
//! layout, no search path, and no directory a command has to be run from, which is also what
//! makes every build and every test work with no network.

/// Every module the library holds, each with the source it is written in.
///
/// The prelude comes first because every other module of it is read with the prelude's names
/// already in scope, which is what `docs/specs/modules.md` says of every module.
const CARRIED: [(&str, &str); 8] = [
    (PRELUDE, include_str!("../../../library/prelude.lm")),
    ("io", include_str!("../../../library/io.lm")),
    ("files", include_str!("../../../library/files.lm")),
    (
        "environment",
        include_str!("../../../library/environment.lm"),
    ),
    ("list", include_str!("../../../library/list.lm")),
    ("strings", include_str!("../../../library/strings.lm")),
    ("map", include_str!("../../../library/map.lm")),
    ("set", include_str!("../../../library/set.lm")),
];

/// The module whose names are in scope everywhere without being imported.
pub const PRELUDE: &str = "prelude";

/// The source an import of `module` reaches, when that name is the library's.
///
/// An import of a library module looks beside no file: a file of that name beside the importing
/// one is not consulted and does not shadow it. The prelude is not among them, because its names
/// are in scope already and an import of it would bring in what is there.
#[must_use]
pub fn imported(module: &str) -> Option<&'static str> {
    if module == PRELUDE {
        return None;
    }
    source_of(module)
}

/// Every module the library holds, in the order it reads them, the prelude first.
pub fn carried() -> impl Iterator<Item = (&'static str, &'static str)> {
    CARRIED.into_iter()
}

/// The source the library module called `module` is written in, when the library holds one.
#[must_use]
pub fn source_of(module: &str) -> Option<&'static str> {
    CARRIED
        .iter()
        .find(|(name, _)| *name == module)
        .map(|(_, source)| *source)
}

//! The modules the compiler declares, which nothing loads from source yet.
//!
//! `docs/specs/io.md` states what each one holds. The library is Lumen source already, which
//! `docs/specs/library.md` states, and these two are not: each reaches a JVM method the language
//! cannot yet name. They become library modules over the `extern` declaration that names one, and
//! nothing a program writes changes when they do.

use crate::types::Type;

/// The modules the compiler declares, rather than leaving them for module loading.
const SUPPLIED: [&str; 2] = ["io", "files"];

/// Whether the compiler supplies `module`, rather than leaving it for loading to find a file.
///
/// Loading asks this before it looks for a file, so an import of one of these names reaches the
/// module declared here rather than a file that is not there.
#[must_use]
pub fn supplies(module: &str) -> bool {
    SUPPLIED.contains(&module)
}

/// The type of `name` inside `module`, when the module declares it.
///
/// What each module holds is this one table, so a name it does not hold is refused here rather
/// than anywhere further in. `docs/specs/io.md` states the same types in the same order.
pub(crate) fn declared(module: &str, name: &str) -> Option<Type> {
    match (module, name) {
        ("io", "print" | "println") => Some(Type::function(vec![Type::string()], Type::Unit)),
        ("files", "read") => Some(Type::function(
            vec![Type::string()],
            Type::result(Type::string(), Type::string()),
        )),
        _ => None,
    }
}

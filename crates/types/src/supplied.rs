//! The modules the compiler declares, which nothing loads from source yet.
//!
//! `docs/specs/io.md` states what each one holds. They are declared here for the same reason the
//! prelude is declared in the resolver: a module cannot be loaded from a file yet, and a program
//! that can show nobody what it worked out is not much of a program. They become ordinary Lumen
//! source once a module can be loaded, and nothing a program writes changes when they do.

use crate::types::Type;

/// The modules the compiler declares, rather than leaving them for module loading.
const SUPPLIED: [&str; 2] = ["io", "files"];

/// Whether the compiler supplies `module`, rather than leaving it for module loading.
pub(crate) fn supplies(module: &str) -> bool {
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

//! The names every module has in scope without importing anything.
//!
//! `docs/specs/modules.md` lists them. They become Lumen source once a module can be loaded;
//! until then this is the prelude.

/// The types the prelude supplies.
pub(crate) const TYPES: [&str; 6] = ["Bool", "Int", "List", "Option", "Result", "String"];

/// The constructors the prelude supplies.
pub(crate) const CONSTRUCTORS: [&str; 4] = ["Err", "None", "Ok", "Some"];

/// The functions the prelude supplies.
///
/// `or` is the total way to get a value out of an `Option`, and `todo` is the hole
/// `docs/specs/holes.md` states.
pub(crate) const FUNCTIONS: [&str; 2] = ["or", "todo"];

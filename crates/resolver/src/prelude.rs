//! The names every module has in scope without importing anything.
//!
//! `docs/specs/modules.md` lists them. They become Lumen source once a module can be loaded;
//! until then this is the prelude.

/// The types the prelude supplies.
pub(crate) const TYPES: [&str; 6] = ["Bool", "Int", "List", "Option", "Result", "String"];

/// The traits the prelude supplies, which `docs/specs/traits.md` lists.
pub(crate) const TRAITS: [&str; 1] = [EQ];

/// The methods those traits declare, each a name in scope beside the functions.
pub(crate) const TRAIT_METHODS: [&str; 1] = [EQUALS];

/// The trait `==` is, which the library will ship once the prelude is Lumen source.
pub(crate) const EQ: &str = "Eq";

/// The one method `Eq` declares.
const EQUALS: &str = "is_equal";

/// The types the prelude has instances of `Eq` for, which `docs/design.md` section 8 names.
pub(crate) const EQUATABLE: [&str; 3] = ["Bool", "Int", "String"];

/// The methods the trait called `name` declares, when the prelude is the one that declares it.
pub(crate) fn methods_of(name: &str) -> Option<&'static [&'static str]> {
    (name == EQ).then_some(&TRAIT_METHODS)
}

/// The constructors the prelude supplies.
pub(crate) const CONSTRUCTORS: [&str; 4] = ["Err", "None", "Ok", "Some"];

/// The functions the prelude supplies.
///
/// `or` is the total way to get a value out of an `Option`, and `todo` is the hole
/// `docs/specs/holes.md` states.
pub(crate) const FUNCTIONS: [&str; 2] = ["or", "todo"];

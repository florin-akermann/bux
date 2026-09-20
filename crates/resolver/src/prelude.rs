//! The names every module has in scope without importing anything.
//!
//! `docs/specs/modules.md` lists them. They become Lumen source once a module can be loaded;
//! until then this is the prelude.

/// The trait `==` is, which `docs/specs/traits.md` writes out.
pub const EQ: &str = "Eq";
/// The method of that trait, which is what `==` is written as.
pub const IS_EQUAL: &str = "is_equal";
/// The trait `+` is.
pub const ADD: &str = "Add";
/// The trait binary `-` is.
pub const SUB: &str = "Sub";
/// The trait `*` is.
pub const MUL: &str = "Mul";
/// The trait `/` is.
pub const DIV: &str = "Div";
/// The trait `%` is.
pub const REM: &str = "Rem";
/// The trait prefix `-` is.
pub const NEG: &str = "Neg";
/// The trait `<`, `<=`, `>`, and `>=` are.
pub const ORD: &str = "Ord";
/// The method of that trait, which is what the four comparisons are written as.
pub const IS_LESS: &str = "is_less";
/// The trait a value opts into a hash with, rather than the `hashCode` a JVM object is born with.
pub const HASH: &str = "Hash";
/// The method of that trait, which gives a value the whole number that stands for what it holds.
pub const HASHED: &str = "hashed";
/// The trait a value opts into text with, rather than the `toString` a JVM object is born with.
pub const SHOW: &str = "Show";
/// The method of that trait, which renders a value as text a reader reads.
pub const SHOWN: &str = "shown";
/// The trait a whole-number literal is, which `docs/specs/literals.md` writes out.
pub const INTEGER_LITERAL: &str = "IntegerLiteral";
/// The method of that trait a whole number is written as, which turns one into the type.
pub const FROM_LITERAL: &str = "from_literal";
/// The method of that trait giving the smallest whole number its type holds.
pub const LOWEST: &str = "lowest";
/// The method of that trait giving the largest whole number its type holds.
pub const HIGHEST: &str = "highest";

/// The types the prelude supplies.
pub(crate) const TYPES: [&str; 6] = ["Bool", "Int", "List", "Option", "Result", "String"];

/// A trait the prelude supplies: what it declares, and which types it already has instances for.
///
/// `docs/specs/traits.md` writes `Eq` out, `docs/specs/operators.md` the seven an operator is,
/// and `docs/specs/literals.md` the one a literal is, each with the instances the library will
/// ship once the prelude is Lumen source.
pub struct Supplied {
    pub name: &'static str,
    pub methods: &'static [&'static str],
    pub instances: &'static [&'static str],
}

/// Every trait the prelude supplies: the four standard ones, the trait each operator is, and the
/// one a literal is.
pub const TRAITS: [Supplied; 11] = [
    Supplied {
        name: EQ,
        methods: &[IS_EQUAL],
        instances: &["Bool", "Int", "String"],
    },
    Supplied {
        name: ADD,
        methods: &["add"],
        instances: &["Int", "String"],
    },
    Supplied {
        name: SUB,
        methods: &["subtract"],
        instances: &["Int"],
    },
    Supplied {
        name: MUL,
        methods: &["multiply"],
        instances: &["Int"],
    },
    Supplied {
        name: DIV,
        methods: &["divide"],
        instances: &["Int"],
    },
    Supplied {
        name: REM,
        methods: &["remainder"],
        instances: &["Int"],
    },
    Supplied {
        name: NEG,
        methods: &["negate"],
        instances: &["Int"],
    },
    Supplied {
        name: ORD,
        methods: &[IS_LESS],
        instances: &["Bool", "Int", "String"],
    },
    Supplied {
        name: HASH,
        methods: &[HASHED],
        instances: &["Bool", "Int", "String"],
    },
    Supplied {
        name: SHOW,
        methods: &[SHOWN],
        instances: &["Bool", "Int", "String"],
    },
    Supplied {
        name: INTEGER_LITERAL,
        methods: &[LOWEST, HIGHEST, FROM_LITERAL],
        instances: &["Int"],
    },
];

/// The traits a type derives, which `docs/specs/derive.md` states are the four standard ones.
///
/// Each one is a reading of what a value holds, so each follows from the declaration and there is
/// nothing for an author to decide; no other trait is derivable.
pub const DERIVABLE: [&str; 4] = [EQ, ORD, HASH, SHOW];

/// The constructors the prelude supplies.
pub(crate) const CONSTRUCTORS: [&str; 4] = ["Err", "None", "Ok", "Some"];

/// The functions the prelude supplies.
///
/// `or` is the total way to get a value out of an `Option`, and `todo` is the hole
/// `docs/specs/holes.md` states.
pub(crate) const FUNCTIONS: [&str; 2] = ["or", "todo"];

/// The trait names, which are names in the type scope beside the types.
pub(crate) fn trait_names() -> Vec<&'static str> {
    TRAITS.iter().map(|supplied| supplied.name).collect()
}

/// The names of every method they declare, which are names in the value scope beside the functions.
pub(crate) fn method_names() -> Vec<&'static str> {
    TRAITS
        .iter()
        .flat_map(|supplied| supplied.methods)
        .copied()
        .collect()
}

/// The one method the trait called `name` declares, where it declares exactly one.
#[must_use]
pub fn method_of(name: &str) -> Option<&'static str> {
    match methods_of(name) {
        Some([only]) => Some(only),
        _ => None,
    }
}

/// The methods the trait called `name` declares, when the prelude is the one that declares it.
#[must_use]
pub fn methods_of(name: &str) -> Option<&'static [&'static str]> {
    TRAITS
        .iter()
        .find(|supplied| supplied.name == name)
        .map(|supplied| supplied.methods)
}

/// The trait the prelude declares `method` in, when the prelude is the one that declares it.
#[must_use]
pub fn trait_of(method: &str) -> Option<&'static str> {
    TRAITS
        .iter()
        .find(|supplied| supplied.methods.contains(&method))
        .map(|supplied| supplied.name)
}

/// Every instance the prelude supplies, as the two names that say which one it is.
pub fn instances() -> impl Iterator<Item = (&'static str, &'static str)> {
    TRAITS.iter().flat_map(|supplied| {
        supplied
            .instances
            .iter()
            .map(|for_type| (supplied.name, *for_type))
    })
}

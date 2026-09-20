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

/// Every trait the prelude supplies: `Eq`, the trait each operator is, and the one a literal is.
pub const TRAITS: [Supplied; 9] = [
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
        methods: &["is_less"],
        instances: &["Int"],
    },
    Supplied {
        name: INTEGER_LITERAL,
        methods: &[LOWEST, HIGHEST, FROM_LITERAL],
        instances: &["Int"],
    },
];

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

//! The names every module has in scope without importing anything.
//!
//! `docs/specs/library.md` says where the source is and what is in it, and
//! `docs/specs/modules.md` lists the names. The compiler carries `library/prelude.lm` and reads
//! what it declares out of it, so `Option` and `Eq` are declared once rather than declared once
//! and tabulated beside it.

use std::sync::LazyLock;

use lumen_ast::{Item, Program};

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

/// The traits a type derives, which `docs/specs/derive.md` states are the four standard ones.
///
/// Each one is a reading of what a value holds, so each follows from the declaration and there is
/// nothing for an author to decide; no other trait is derivable.
pub const DERIVABLE: [&str; 4] = [EQ, ORD, HASH, SHOW];

/// The types the JVM holds directly, with how many arguments each is written with.
///
/// `docs/specs/library.md` draws the line here: what a type is made of is the JVM for these four
/// and a declaration for every other, and a program cannot tell which it has. No Lumen
/// declaration could write them, which is why they are the one part of the prelude left as a
/// table rather than read out of `library/prelude.lm`.
const HELD: [(&str, usize); 4] = [("Bool", 0), ("Int", 0), (LIST_TYPE, 1), ("String", 0)];

/// The type a list is, which the prelude writes the instances of and no module declares.
pub const LIST_TYPE: &str = "List";

/// The functions the compiler supplies, which is `todo` because a hole has no body to be written.
const SUPPLIED: [&str; 1] = ["todo"];

/// The one library module the compiler holds a function of, which `docs/specs/library.md` names.
pub const LIST: &str = "list";

/// The one that grows a list, which no expression the grammar writes builds.
pub const PUSH: &str = "push";

/// The one that reads a list at an index, which the JVM reads in the time it reads one slot in.
pub const AT: &str = "at";

/// The two functions the compiler holds, which `docs/specs/library.md` says no source writes.
const HELD_FUNCTIONS: [&str; 2] = [PUSH, AT];

/// The ones the module called `module` offers to whatever imports it.
///
/// They are `list`'s names although the compiler holds them, so `list` offers them as it offers
/// `length`, and a module reaches them by importing `list` and writing `list.push`.
#[must_use]
pub fn offered_by(module: &str) -> Vec<&'static str> {
    if module == LIST {
        return HELD_FUNCTIONS.to_vec();
    }
    Vec::new()
}

/// The ones the source of the module called `module` writes without naming a module first.
///
/// The prelude writes the instances of `List` and reads a list with `at`, and it imports nothing,
/// so it is the one module whose own source writes them. A program module called `list` is a
/// program module like any other and gets neither name.
#[must_use]
pub fn held_for(module: &str) -> Vec<&'static str> {
    if module == crate::library::PRELUDE {
        return HELD_FUNCTIONS.to_vec();
    }
    Vec::new()
}

/// A trait the prelude supplies: what it declares, and which types it already has instances for.
pub struct Supplied {
    pub name: &'static str,
    pub methods: Vec<&'static str>,
    pub instances: Vec<&'static str>,
}

/// The prelude as the compiler carries it, parsed.
pub(crate) fn program() -> &'static Program {
    carried()
}

/// Those same types, each with how many arguments it is written with.
pub fn held_types() -> impl Iterator<Item = (&'static str, usize)> {
    HELD.into_iter()
}

/// The functions in scope while the prelude itself is resolved, which is `todo` and nothing else.
pub(crate) fn supplied() -> Vec<&'static str> {
    SUPPLIED.to_vec()
}

/// The types in scope everywhere: the four the JVM holds, and the ones the prelude declares.
pub(crate) fn types() -> Vec<&'static str> {
    let mut found = held();
    found.extend(carried().items.iter().filter_map(|item| match item {
        Item::Type(declaration) => Some(declaration.name.text.as_str()),
        _ => None,
    }));
    found
}

/// The names in scope while the prelude itself is resolved, which is what the JVM holds.
///
/// The prelude declares what every other module's scope is seeded with, so it cannot be resolved
/// against that scope: every name it writes would already be in it.
pub(crate) fn held() -> Vec<&'static str> {
    HELD.iter().map(|(name, _)| *name).collect()
}

/// Whether the prelude declares a type of this name, which is what puts it in the prelude's own
/// package rather than a module's.
#[must_use]
pub fn declares_type(name: &str) -> bool {
    declared_types().any(|declaration| declaration.name.text == name)
}

/// Whether the prelude declares a trait of this name, which is a trait every module reaches.
///
/// A trait a module declares is that module's own, which `docs/specs/modules.md` states, so a
/// trait two modules both name is one of these and no other.
#[must_use]
pub fn declares_trait(name: &str) -> bool {
    traits().any(|declaration| declaration.name.text == name)
}

/// The constructors in scope everywhere, which are the variants of the types the prelude declares.
pub(crate) fn constructors() -> Vec<&'static str> {
    declared_types()
        .flat_map(|declaration| variants_of(&declaration.definition))
        .collect()
}

/// The functions in scope everywhere: the ones the prelude declares, and `todo`.
pub(crate) fn functions() -> Vec<&'static str> {
    let mut found: Vec<&'static str> = carried()
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(function) => Some(function.name.text.as_str()),
            _ => None,
        })
        .collect();
    found.extend(SUPPLIED);
    found
}

/// The trait names, which are names in the type scope beside the types.
pub(crate) fn trait_names() -> Vec<&'static str> {
    traits()
        .map(|declaration| declaration.name.text.as_str())
        .collect()
}

/// The names of every method they declare, which are names in the value scope beside the functions.
pub(crate) fn method_names() -> Vec<&'static str> {
    traits()
        .flat_map(|declaration| &declaration.methods)
        .map(|signature| signature.name.text.as_str())
        .collect()
}

/// Every trait the prelude declares, with its methods and the types it has an instance for.
#[must_use]
pub fn supplied_traits() -> Vec<Supplied> {
    traits()
        .map(|declaration| {
            let name = declaration.name.text.as_str();
            Supplied {
                name,
                methods: declaration
                    .methods
                    .iter()
                    .map(|signature| signature.name.text.as_str())
                    .collect(),
                instances: instances()
                    .filter(|(of, _)| *of == name)
                    .map(|(_, for_type)| for_type)
                    .collect(),
            }
        })
        .collect()
}

/// The one method the trait called `name` declares, where it declares exactly one.
#[must_use]
pub fn method_of(name: &str) -> Option<&'static str> {
    match methods_of(name).as_deref() {
        Some([only]) => Some(only),
        _ => None,
    }
}

/// The methods the trait called `name` declares, when the prelude is the one that declares it.
#[must_use]
pub fn methods_of(name: &str) -> Option<Vec<&'static str>> {
    traits()
        .find(|declaration| declaration.name.text == name)
        .map(|declaration| {
            declaration
                .methods
                .iter()
                .map(|signature| signature.name.text.as_str())
                .collect()
        })
}

/// The trait the prelude declares `method` in, when the prelude is the one that declares it.
#[must_use]
pub fn trait_of(method: &str) -> Option<&'static str> {
    traits()
        .find(|declaration| {
            declaration
                .methods
                .iter()
                .any(|signature| signature.name.text == method)
        })
        .map(|declaration| declaration.name.text.as_str())
}

/// Every instance the prelude has, as the two names that say which one it is, in the order the
/// library writes them.
pub fn instances() -> impl Iterator<Item = (&'static str, &'static str)> {
    carried().items.iter().filter_map(|item| match item {
        Item::Instance(declaration) => Some((
            declaration.trait_name.text.as_str(),
            declaration.for_type.text.as_str(),
        )),
        _ => None,
    })
}

/// Every trait the prelude declares, in the order it writes them.
fn traits() -> impl Iterator<Item = &'static lumen_ast::TraitDeclaration> {
    carried().items.iter().filter_map(|item| match item {
        Item::Trait(declaration) => Some(declaration),
        _ => None,
    })
}

/// Every type the prelude declares, in the order it writes them.
fn declared_types() -> impl Iterator<Item = &'static lumen_ast::TypeDeclaration> {
    carried().items.iter().filter_map(|item| match item {
        Item::Type(declaration) => Some(declaration),
        _ => None,
    })
}

/// The names of the variants a type declaration writes, which a record declaration has none of.
fn variants_of(definition: &'static lumen_ast::TypeDefinition) -> Vec<&'static str> {
    match definition {
        lumen_ast::TypeDefinition::Foreign { .. } | lumen_ast::TypeDefinition::Record(_) => {
            Vec::new()
        }
        lumen_ast::TypeDefinition::Variants(variants) => variants
            .iter()
            .map(|variant| variant.name.text.as_str())
            .collect(),
    }
}

/// The prelude, parsed once.
///
/// A library that does not parse is the compiler's own failure and not any program's, which
/// `docs/specs/library.md` states; the compiler's own tests are where it is caught.
fn carried() -> &'static Program {
    static PARSED: LazyLock<Program> = LazyLock::new(|| {
        let source = crate::library::source_of(crate::library::PRELUDE)
            .expect("the library the compiler carries holds the prelude");
        lumen_parser::parse(source).expect("the library the compiler carries parses")
    });
    &PARSED
}

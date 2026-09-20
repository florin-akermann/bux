//! The invariants of `docs/specs/api-surface.md`, checked on generated modules.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ast::{Item, TypeDefinition};

use crate::common::{page, tree};

/// Type declarations, each as canonical form writes it, each numbered apart, and none commented.
const TYPES: [&str; 4] = [
    "type Basket{n} = {\n    apples: Int\n    pears: Int\n}",
    "type Order{n} =\n    | Unplaced{n}\n    | Placed{n}(Int)",
    "type Held{n} = Held{n}(String)",
    "type Pair{n}<A, B> = {\n    left: A\n    right: B\n}",
];

/// Functions, each self-contained, so any of them may be written beside any other.
const FUNCTIONS: [&str; 4] = [
    "fn answer{n}() -> Int {\n    7\n}",
    "fn named{n}(word: String) -> String {\n    word\n}",
    "fn shared{n}(total: Int, people: Int) -> Int {\n    or(total / people, 0)\n}",
    "fn kept{n}<A>(value: A) -> A {\n    value\n}",
];

#[hegel::test]
fn a_module_of_types_alone_holding_no_comment_is_its_own_page(tc: TestCase) {
    let source = module(&drawn(&tc, &TYPES));

    assert_eq!(page(&source), source, "{source}");
}

#[hegel::test]
fn every_name_a_module_declares_is_on_its_page(tc: TestCase) {
    let source = module(&anything(&tc));
    let page = page(&source);

    for declared in declarations(&source) {
        assert!(
            page.contains(&declared),
            "{declared} is missing from {page}"
        );
    }
}

#[hegel::test]
fn printing_a_page_is_deterministic(tc: TestCase) {
    let source = module(&anything(&tc));

    assert_eq!(page(&source), page(&source), "{source}");
}

/// Declarations of both kinds, which is what an ordinary module is written out of.
fn anything(tc: &TestCase) -> Vec<String> {
    let mut written = drawn(tc, &FUNCTIONS);
    written.extend(drawn(tc, &TYPES));
    written
}

/// Zero or more of `written`, each numbered by where it is drawn so that two stay apart.
fn drawn(tc: &TestCase, written: &[&str]) -> Vec<String> {
    tc.draw(gs::vecs(gs::sampled_from(written)).max_size(4))
        .iter()
        .enumerate()
        .map(|(at, one)| one.replace("{n}", &at.to_string()))
        .collect()
}

/// The source of a module of `declarations`, in canonical form, which is empty when there are none.
fn module(declarations: &[String]) -> String {
    if declarations.is_empty() {
        return String::new();
    }
    declarations.join("\n\n") + "\n"
}

/// Every name `source` declares at the top level, which is what a page holds.
fn declarations(source: &str) -> Vec<String> {
    let mut declared = Vec::new();
    for item in &tree(source).items {
        match item {
            // An import names another module, and an instance declares no name of its own.
            Item::Import(_) | Item::Instance(_) => {}
            Item::Function(function) => declared.push(function.name.text.clone()),
            Item::Type(declaration) => {
                declared.push(declaration.name.text.clone());
                declared.extend(variants(&declaration.definition));
            }
            Item::Trait(declaration) => {
                declared.push(declaration.name.text.clone());
                let methods = declaration.methods.iter();
                declared.extend(methods.map(|method| method.name.text.clone()));
            }
        }
    }
    declared
}

/// The variants a definition declares, which a record definition has none of.
fn variants(definition: &TypeDefinition) -> Vec<String> {
    let TypeDefinition::Variants(declared) = definition else {
        return Vec::new();
    };
    declared
        .iter()
        .map(|variant| variant.name.text.clone())
        .collect()
}

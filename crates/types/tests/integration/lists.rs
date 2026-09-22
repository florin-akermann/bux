//! `docs/specs/library.md`: the two functions of `list` the compiler holds rather than declares.
//!
//! `library/list.lm` writes neither `push` nor `at`, because neither can be said in Bux, so the
//! module offers two names its own source never names. A module importing `list` reaches them
//! exactly as it reaches the three the source does write.

use lumen_ast::Item;
use lumen_resolver::library;
use lumen_types::Imported;

use crate::common::{inferred, inferred_type_reaching, refusal_reaching};

/// The module the compiler holds the two functions for.
const LIST: &str = "list";

#[test]
fn push_takes_the_list_and_a_value_of_what_it_holds_and_gives_back_the_list() {
    let source = reaching("_ = list.push([1, 2], 3)");

    assert_eq!(
        inferred_type_reaching(&source, "list.push", 1, &offered()),
        "(List<Int>, Int) -> List<Int>"
    );
}

#[test]
fn at_takes_the_list_and_an_index_and_gives_back_what_it_may_hold_there() {
    let source = reaching("_ = list.at([\"one\"], 0)");

    assert_eq!(
        inferred_type_reaching(&source, "list.at", 1, &offered()),
        "(List<String>, Int) -> Option<String>"
    );
}

#[test]
fn a_value_of_another_type_than_the_list_holds_is_refused_by_push() {
    let source = reaching("_ = list.push([1, 2], \"three\")");

    assert_eq!(
        refusal_reaching(&source, &offered()).message(),
        "expected `Int`, found `String`"
    );
}

#[test]
fn an_index_that_is_no_whole_number_is_refused_by_at() {
    let source = reaching("_ = list.at([1], \"first\")");

    assert_eq!(
        refusal_reaching(&source, &offered()).message(),
        "expected `Int`, found `String`"
    );
}

#[test]
fn the_source_of_the_list_module_declares_neither_of_them() {
    let declared = declared_by_the_source();

    assert!(!declared.contains(&"push".to_owned()));
    assert!(!declared.contains(&"at".to_owned()));
}

/// What `list` offers, which is what its source declares and the two the compiler holds.
fn offered() -> Imported {
    let source = library::source_of(LIST).expect("the library carries the `list` module");
    Imported::default().offering(LIST, inferred(source).surface().clone())
}

/// A module importing `list` and writing `written` as the body of `main`.
fn reaching(written: &str) -> String {
    format!("import list\n\nfn main() -> () {{\n    {written}\n}}\n")
}

/// Every function `library/list.lm` writes, which is what the module's own source declares.
fn declared_by_the_source() -> Vec<String> {
    let source = library::source_of(LIST).expect("the library carries the `list` module");
    inferred(source)
        .resolved()
        .program()
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(function) => Some(function.name.text.clone()),
            _ => None,
        })
        .collect()
}

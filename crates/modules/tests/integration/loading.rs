//! What loading reads, and the order it hands the modules over in.
//!
//! `docs/specs/modules.md` states the rules these hold: an import names the file beside the one
//! that writes it, a module is read once however many modules import it, and a module is handed
//! over below everything it imports.

use lumen_modules::{Loaded, Module, load};

use crate::common::Beside;

/// The names of every module loaded, in the order they are handed over.
fn order_of(loaded: &Loaded) -> Vec<&str> {
    loaded.modules().iter().map(Module::name).collect()
}

#[test]
fn a_module_that_imports_nothing_is_the_one_module_loaded() {
    let beside = Beside::holding(&[("main", "fn main() -> () {\n}\n")]);
    let loaded = load(&beside.file_of("main")).expect("a module of its own loads");
    assert_eq!(order_of(&loaded), ["main"]);
}

#[test]
fn a_module_is_named_by_its_file_and_carries_the_text_it_was_read_from() {
    let source = "fn main() -> () {\n}\n";
    let beside = Beside::holding(&[("main", source)]);
    let loaded = load(&beside.file_of("main")).expect("a module of its own loads");
    let module = &loaded.modules()[0];
    assert_eq!(module.name(), "main");
    assert_eq!(module.source(), source);
    assert_eq!(module.path(), beside.file_of("main"));
}

#[test]
fn an_import_names_the_file_beside_the_one_that_writes_it() {
    let beside = Beside::holding(&[
        ("main", "import greeting\n\nfn main() -> () {\n}\n"),
        ("greeting", "fn hello() -> String {\n    \"hi\"\n}\n"),
    ]);
    let loaded = load(&beside.file_of("main")).expect("a module beside it loads");
    assert_eq!(order_of(&loaded), ["greeting", "main"]);
}

#[test]
fn a_module_is_handed_over_below_everything_it_imports() {
    let beside = Beside::holding(&[
        ("main", "import middle\n\nfn main() -> () {\n}\n"),
        ("middle", "import bottom\n\nfn held() -> Int {\n    1\n}\n"),
        ("bottom", "fn deepest() -> Int {\n    1\n}\n"),
    ]);
    let loaded = load(&beside.file_of("main")).expect("a chain of modules loads");
    assert_eq!(order_of(&loaded), ["bottom", "middle", "main"]);
}

#[test]
fn a_module_two_modules_import_is_read_once() {
    let beside = Beside::holding(&[
        (
            "main",
            "import left\n\nimport right\n\nfn main() -> () {\n}\n",
        ),
        ("left", "import shared\n\nfn one() -> Int {\n    1\n}\n"),
        ("right", "import shared\n\nfn two() -> Int {\n    2\n}\n"),
        ("shared", "fn held() -> Int {\n    3\n}\n"),
    ]);
    let loaded = load(&beside.file_of("main")).expect("a module two modules import loads");
    assert_eq!(order_of(&loaded), ["shared", "left", "right", "main"]);
}

#[test]
fn the_module_the_command_named_is_the_one_handed_over_last() {
    let beside = Beside::holding(&[
        (
            "greeting",
            "import shared\n\nfn hello() -> Int {\n    1\n}\n",
        ),
        ("shared", "fn held() -> Int {\n    1\n}\n"),
    ]);
    let loaded = load(&beside.file_of("greeting")).expect("a module that is not a program loads");
    assert_eq!(order_of(&loaded), ["shared", "greeting"]);
}

#[test]
fn the_tree_of_a_loaded_module_is_the_one_the_grammar_made_of_it() {
    let beside = Beside::holding(&[
        ("main", "import greeting\n\nfn main() -> () {\n}\n"),
        ("greeting", "fn hello() -> Int {\n    1\n}\n"),
    ]);
    let loaded = load(&beside.file_of("main")).expect("a module beside it loads");
    let greeting = &loaded.modules()[0];
    let expected = lumen_parser::parse(greeting.source()).expect("a well-formed module parses");
    assert_eq!(greeting.program(), &expected);
}

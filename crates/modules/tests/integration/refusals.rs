//! What loading will not have, and what each refusal says.
//!
//! `docs/specs/modules.md` states them: an import that names no file is `L0306`, and a ring of
//! imports is `L0307`. Both point at the import being followed, in the file that wrote it.

use lumen_diagnostics::Code;
use lumen_modules::{NotLoaded, load};

use crate::common::Beside;

/// The refusal `path` is loaded with, which every one of these expects there to be.
fn refused(beside: &Beside, named: &str) -> (Code, String, std::path::PathBuf) {
    match load(&beside.file_of(named)) {
        Ok(_) => panic!("{named} is refused"),
        Err(NotLoaded::Unreadable { path, why }) => {
            panic!("{} is readable: {why}", path.display())
        }
        Err(NotLoaded::Refused(refused)) => (
            refused.diagnostic().code(),
            refused.diagnostic().message().to_owned(),
            refused.path().to_path_buf(),
        ),
    }
}

#[test]
fn an_import_that_names_no_file_beside_it_is_refused() {
    let beside = Beside::holding(&[("main", "import greeting\n\nfn main() -> () {\n}\n")]);
    let (code, message, path) = refused(&beside, "main");
    assert_eq!(code, Code::NoSuchModule);
    assert_eq!(message, "there is no module named `greeting`");
    assert_eq!(path, beside.file_of("main"));
}

#[test]
fn a_missing_module_is_refused_in_the_file_that_imports_it_rather_than_the_one_named() {
    let beside = Beside::holding(&[
        ("main", "import middle\n\nfn main() -> () {\n}\n"),
        ("middle", "import bottom\n\nfn held() -> Int {\n    1\n}\n"),
    ]);
    let (code, _, path) = refused(&beside, "main");
    assert_eq!(code, Code::NoSuchModule);
    assert_eq!(path, beside.file_of("middle"));
}

#[test]
fn the_refusal_points_at_the_import_that_names_the_missing_module() {
    let source = "import greeting\n\nfn main() -> () {\n}\n";
    let beside = Beside::holding(&[("main", source)]);
    let Err(NotLoaded::Refused(refused)) = load(&beside.file_of("main")) else {
        panic!("a missing module is refused");
    };
    assert_eq!(refused.diagnostic().span().text(source), "greeting");
}

#[test]
fn two_modules_that_import_each_other_are_refused() {
    let beside = Beside::holding(&[
        ("main", "import greeting\n\nfn main() -> () {\n}\n"),
        ("greeting", "import main\n\nfn hello() -> Int {\n    1\n}\n"),
    ]);
    let (code, message, path) = refused(&beside, "main");
    assert_eq!(code, Code::RingOfImports);
    assert_eq!(message, "`main` imports `greeting`, which imports `main`");
    assert_eq!(path, beside.file_of("greeting"));
}

#[test]
fn a_module_that_imports_itself_is_refused() {
    let beside = Beside::holding(&[("main", "import main\n\nfn main() -> () {\n}\n")]);
    let (code, message, _) = refused(&beside, "main");
    assert_eq!(code, Code::RingOfImports);
    assert_eq!(message, "`main` imports `main`");
}

#[test]
fn a_longer_ring_is_named_all_the_way_round() {
    let beside = Beside::holding(&[
        ("main", "import middle\n\nfn main() -> () {\n}\n"),
        ("middle", "import bottom\n\nfn held() -> Int {\n    1\n}\n"),
        ("bottom", "import main\n\nfn deepest() -> Int {\n    1\n}\n"),
    ]);
    let (code, message, _) = refused(&beside, "main");
    assert_eq!(code, Code::RingOfImports);
    assert_eq!(
        message,
        "`main` imports `middle`, which imports `bottom`, which imports `main`"
    );
}

#[test]
fn a_module_that_two_modules_import_is_no_ring() {
    let beside = Beside::holding(&[
        (
            "main",
            "import left\n\nimport right\n\nfn main() -> () {\n}\n",
        ),
        ("left", "import shared\n\nfn one() -> Int {\n    1\n}\n"),
        ("right", "import shared\n\nfn two() -> Int {\n    2\n}\n"),
        ("shared", "fn held() -> Int {\n    3\n}\n"),
    ]);
    load(&beside.file_of("main")).expect("two modules importing one is no ring");
}

#[test]
fn a_module_the_grammar_refuses_is_refused_against_its_own_file() {
    let beside = Beside::holding(&[
        ("main", "import greeting\n\nfn main() -> () {\n}\n"),
        ("greeting", "fn hello( -> Int {\n}\n"),
    ]);
    let (code, _, path) = refused(&beside, "main");
    assert_eq!(code, Code::UnexpectedToken);
    assert_eq!(path, beside.file_of("greeting"));
}

#[test]
fn a_file_that_is_not_there_at_all_is_no_program_to_refuse() {
    let beside = Beside::holding(&[]);
    let missing = beside.directory().join("nothing.lm");
    let Err(NotLoaded::Unreadable { path, .. }) = load(&missing) else {
        panic!("a file that is not there cannot be read");
    };
    assert_eq!(path, missing);
}

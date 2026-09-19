//! `lumen api`: the page a module has, as the command line prints it.

use crate::common::{Example, lumen};

/// A module with a function, an algebraic data type, and a record.
const MODULE: &str = "fn told(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Failed(reason) => reason\n    }\n}\n\ntype Payment =\n    | Pending\n    | Failed(String)\n\ntype User = {\n    id: Int\n}\n";

#[test]
fn api_prints_every_name_the_module_declares_with_the_type_it_has() {
    let example = Example::new(MODULE);

    let run = lumen(&["api", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(
        run.stdout,
        concat!(
            "fn told(payment: Payment) -> String\n\n",
            "type Payment =\n    | Pending\n    | Failed(String)\n\n",
            "type User = {\n    id: Int\n}\n"
        )
    );
}

#[test]
fn the_page_goes_to_standard_output_and_a_refusal_to_standard_error() {
    let example = Example::new("fn told() -> Int {\n    missing()\n}\n");

    let run = lumen(&["api", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 1);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.starts_with("error[L0300]: "), "{}", run.stderr);
}

#[test]
fn a_module_that_holds_a_hole_has_a_page_because_a_hole_is_well_typed() {
    let example = Example::new("fn counted() -> Int {\n    todo(\"count them\")\n}\n");

    let run = lumen(&["api", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "fn counted() -> Int\n");
}

#[test]
fn a_file_that_cannot_be_read_is_neither_a_page_nor_a_refusal() {
    let run = lumen(&["api", "no/such/file.lm"]);

    assert_eq!(run.code, 2);
    assert_eq!(run.stdout, "");
}

#[test]
fn api_writes_nothing_beside_the_source() {
    let example = Example::new(MODULE);

    lumen(&["api", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(example.beside("example.class"), None);
}

#[test]
fn the_help_of_api_says_what_the_page_holds() {
    let run = lumen(&["help", "api"]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(
        run.stdout.contains("every name the file declares"),
        "{}",
        run.stdout
    );
}

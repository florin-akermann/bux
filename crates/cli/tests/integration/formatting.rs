//! `lumen fmt` and `lumen check`: the canonical-form gate as the command line sees it.

use crate::common::{Example, lumen};

#[test]
fn fmt_rewrites_a_file_that_is_not_in_canonical_form() {
    let example = Example::new("fn f() {\n  a  :=  1\n}\n");
    let run = lumen(&["fmt", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(example.content(), "fn f() {\n    a := 1\n}\n");
}

#[test]
fn fmt_leaves_a_canonical_file_exactly_as_it_is() {
    let example = Example::new("import io\n");
    let run = lumen(&["fmt", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(example.content(), "import io\n");
}

#[test]
fn fmt_writes_nothing_when_the_file_does_not_parse() {
    let example = Example::new("fn (x) {\n}\n");
    let run = lumen(&["fmt", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 1);
    assert!(
        run.stderr
            .contains("error[L0100]: expected a function name, found `(`"),
        "{}",
        run.stderr
    );
    assert_eq!(example.content(), "fn (x) {\n}\n");
}

#[test]
fn check_accepts_a_file_in_canonical_form() {
    let example = Example::new("import io\n");
    let run = lumen(&["check", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.stderr, "");
}

#[test]
fn check_names_the_first_line_that_is_not_in_canonical_form() {
    let example = Example::new("fn f() {\n  a  :=  1\n}\n");
    let run = lumen(&["check", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 1);
    assert!(
        run.stderr
            .contains("error[L0200]: this line is not in canonical form"),
        "{}",
        run.stderr
    );
    assert!(
        run.stderr
            .contains("help: canonical form writes `    a := 1`"),
        "{}",
        run.stderr
    );
    assert_eq!(example.content(), "fn f() {\n  a  :=  1\n}\n");
}

#[test]
fn a_file_that_cannot_be_read_is_not_about_the_program_in_it() {
    let run = lumen(&["check", "no/such/file.lm"]);

    assert_eq!(run.code, 2);
    assert!(
        run.stderr.starts_with("error: no/such/file.lm:"),
        "{}",
        run.stderr
    );
}

#[test]
fn each_command_has_a_help_topic_of_its_own() {
    for command in ["fmt", "check", "explain"] {
        let run = lumen(&["help", command]);
        assert_eq!(run.code, 0, "{}", run.stderr);
        assert!(
            run.stdout.contains("Exit codes:"),
            "{command}: {}",
            run.stdout
        );
    }
}

#[test]
fn explain_prints_the_long_form_of_a_code() {
    let run = lumen(&["explain", "L0105"]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(
        run.stdout
            .starts_with("# L0105 — comparisons are chained\n"),
        "{}",
        run.stdout
    );
    assert!(run.stdout.contains("a < b && b < c"), "{}", run.stdout);
}

#[test]
fn explain_refuses_a_code_the_compiler_cannot_raise() {
    let run = lumen(&["explain", "L9999"]);

    assert_eq!(run.code, 2);
    assert_eq!(run.stderr, "error: there is no diagnostic L9999\n");
    assert_eq!(run.stdout, "");
}

#[test]
fn a_refusal_points_at_the_file_it_was_given() {
    let example = Example::new("fn f() {\n  a  :=  1\n}\n");
    let path = example.path.to_str().expect("a UTF-8 path");
    let run = lumen(&["check", path]);

    assert!(
        run.stderr.contains(&format!("--> {path}:2:1")),
        "{}",
        run.stderr
    );
}

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
        run.stderr.contains("expected a function name, found `(`"),
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
        run.stderr.contains("line 2 is not in canonical form"),
        "{}",
        run.stderr
    );
    assert!(
        run.stderr.contains("canonical:     a := 1"),
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
    for command in ["fmt", "check"] {
        let run = lumen(&["help", command]);
        assert_eq!(run.code, 0, "{}", run.stderr);
        assert!(
            run.stdout.contains("canonical form"),
            "{command}: {}",
            run.stdout
        );
    }
}

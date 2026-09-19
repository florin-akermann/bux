//! `check`: whether a file is in canonical form, and what it says when it is not.
//!
//! Everything it refuses goes out as a diagnostic, so these tests read as the text the author
//! sees. `docs/specs/diagnostics.md` states that layout.

use lumen_diagnostics::render;
use lumen_format::{CheckError, check};

/// The diagnostic `check` refuses `source` with, rendered as the author sees it.
fn refusal(source: &str) -> String {
    match check(source) {
        Ok(()) => panic!("{source:?} is canonical"),
        Err(error) => render(&error.diagnostic(), source, "demo.lm"),
    }
}

#[test]
fn a_canonical_file_passes() {
    assert_eq!(check("import io\n"), Ok(()));
    assert_eq!(check(""), Ok(()));
}

#[test]
fn a_deviation_names_the_line_and_what_belongs_there() {
    assert_eq!(
        refusal("fn f() {\n  a  :=  1\n}\n"),
        concat!(
            "error[L0200]: this line is not in canonical form\n",
            "  --> demo.lm:2:1\n",
            "\n",
            "  2 |   a  :=  1\n",
            "    | ^^^^^^^^^^\n",
            "\n",
            "help: canonical form writes `    a := 1`\n",
        )
    );
}

#[test]
fn a_file_with_a_line_canonical_form_does_not_write_says_so() {
    assert_eq!(
        refusal("import io\n\n\n\n"),
        concat!(
            "error[L0200]: canonical form does not write this line\n",
            "  --> demo.lm:2:1\n",
            "\n",
            "  2 | \n",
            "    | ^\n",
            "\n",
            "help: run `lumen fmt` to take this line out\n",
        )
    );
}

#[test]
fn a_line_where_canonical_form_writes_a_blank_one_says_it_writes_nothing() {
    let refused = refusal("import io\nimport files\n");

    assert!(
        refused.ends_with("help: canonical form writes ``\n"),
        "{refused}"
    );
}

#[test]
fn a_file_that_ends_without_its_newline_says_so() {
    assert_eq!(
        refusal("import io"),
        concat!(
            "error[L0200]: this file does not end the way canonical form ends it\n",
            "  --> demo.lm:1:1\n",
            "\n",
            "  1 | import io\n",
            "    | ^^^^^^^^^\n",
            "\n",
            "help: end the file with a newline\n",
        )
    );
}

#[test]
fn a_file_written_with_windows_line_endings_is_reported_at_its_first_line() {
    assert!(
        refusal("import io\r\n")
            .starts_with("error[L0200]: this line is not in canonical form\n  --> demo.lm:1:1\n"),
        "{}",
        refusal("import io\r\n")
    );
}

#[test]
fn a_file_that_does_not_parse_is_reported_as_the_parse_error() {
    assert_eq!(
        check("fn (x) {\n}\n"),
        Err(CheckError::Parse(
            lumen_parser::parse("fn (x) {\n}\n").expect_err("does not parse")
        ))
    );
    assert_eq!(
        refusal("fn (x) {\n}\n"),
        concat!(
            "error[L0100]: expected a function name, found `(`\n",
            "  --> demo.lm:1:4\n",
            "\n",
            "  1 | fn (x) {\n",
            "    |    ^\n",
            "\n",
            "help: every function has a name; Lumen has no anonymous functions\n",
        )
    );
}

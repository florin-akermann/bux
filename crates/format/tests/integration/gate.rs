//! `check`: whether a file is in canonical form, and what it says when it is not.

use lumen_format::{CheckError, check};

/// The message `check` reports for `source`, which must not be canonical.
fn refusal(source: &str) -> String {
    match check(source) {
        Ok(()) => panic!("{source:?} is canonical"),
        Err(error) => error.to_string(),
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
            "line 2 is not in canonical form\n",
            "  found:       a  :=  1\n",
            "  canonical:     a := 1",
        )
    );
}

#[test]
fn a_file_with_a_line_canonical_form_does_not_write_says_so() {
    assert_eq!(
        refusal("import io\n\n\n\n"),
        "line 2 is not part of canonical form: "
    );
}

#[test]
fn a_blank_line_where_canonical_form_writes_none_names_that_line() {
    assert_eq!(
        refusal("import io\nimport files\n"),
        concat!(
            "line 2 is not in canonical form\n",
            "  found:     import files\n",
            "  canonical: ",
        )
    );
}

#[test]
fn a_file_that_ends_without_its_newline_says_so() {
    assert_eq!(
        refusal("import io"),
        "the file does not end the way canonical form ends it"
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
        "expected a function name, found `(`"
    );
}

#[test]
fn a_file_written_with_windows_line_endings_is_reported_at_its_first_line() {
    assert_eq!(
        refusal("import io\r\n"),
        concat!(
            "line 1 is not in canonical form\n",
            "  found:     import io\r\n",
            "  canonical: import io",
        )
    );
}

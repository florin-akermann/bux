//! Comments: every one of them survives, on a line of its own, above the line it was written on.

use crate::common::{formatted, in_function};

#[test]
fn a_comment_on_its_own_line_stays_where_it_is() {
    assert_eq!(formatted("// why\nimport io\n"), "// why\nimport io\n");
}

#[test]
fn a_comment_at_the_end_of_a_line_moves_above_that_line() {
    assert_eq!(
        in_function("total := 0 // start at nothing"),
        ["// start at nothing", "total := 0"]
    );
}

#[test]
fn a_comment_is_indented_like_the_line_below_it() {
    assert_eq!(
        in_function("for {\n// around again\nbreak\n}"),
        ["for {", "    // around again", "    break", "}"]
    );
}

#[test]
fn a_comment_after_the_last_statement_sits_above_the_closing_brace() {
    assert_eq!(
        in_function("a := 1\n// nothing more\n"),
        ["a := 1", "// nothing more"]
    );
}

#[test]
fn a_comment_after_the_last_item_sits_below_one_blank_line() {
    assert_eq!(
        formatted("import io\n// that is all\n"),
        "import io\n\n// that is all\n"
    );
}

#[test]
fn a_comment_in_a_file_of_nothing_else_is_the_whole_file() {
    assert_eq!(formatted("// only this\n"), "// only this\n");
}

#[test]
fn a_comment_between_record_fields_keeps_its_place() {
    assert_eq!(
        formatted("type User = {\n// who they are\nid: UserId\nactive: Bool\n}\n"),
        "type User = {\n    // who they are\n    id: UserId\n    active: Bool\n}\n"
    );
}

#[test]
fn a_comment_between_variants_keeps_its_place() {
    assert_eq!(
        formatted("type P =\n| Pending\n// and then\n| Done\n"),
        "type P =\n    | Pending\n    // and then\n    | Done\n"
    );
}

#[test]
fn a_comment_between_match_arms_keeps_its_place() {
    assert_eq!(
        in_function("match p {\nA => 1\n// or else\nB => 2\n}"),
        [
            "match p {",
            "    A => 1",
            "    // or else",
            "    B => 2",
            "}"
        ]
    );
}

#[test]
fn a_trailing_blank_is_taken_off_a_comment() {
    assert_eq!(
        formatted("// spaced   \nimport io\n"),
        "// spaced\nimport io\n"
    );
}

#[test]
fn a_comment_inside_a_joined_line_moves_below_it() {
    assert_eq!(
        in_function("save(\n// why\na\n)\nb"),
        ["save(a)", "// why", "b"]
    );
    assert_eq!(in_function("save(\n// why\na\n)"), ["save(a)", "// why"]);
}

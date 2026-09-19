//! Statements: how canonical form writes each line of a block.

use crate::common::{formatted, in_function};

#[test]
fn a_binding_is_spaced_around_its_joiner() {
    assert_eq!(in_function("total:=0"), ["total := 0"]);
    assert_eq!(in_function("var total=0"), ["var total = 0"]);
}

#[test]
fn an_assignment_is_spaced_around_its_operator() {
    assert_eq!(in_function("total=1"), ["total = 1"]);
    assert_eq!(in_function("total+=1"), ["total += 1"]);
}

#[test]
fn a_return_carries_a_value_or_nothing() {
    assert_eq!(in_function("return"), ["return"]);
    assert_eq!(in_function("return  total"), ["return total"]);
}

#[test]
fn a_break_and_a_continue_are_written_alone() {
    assert_eq!(
        in_function("for {\nbreak\ncontinue\n}"),
        ["for {", "    break", "    continue", "}",]
    );
}

#[test]
fn a_for_writes_the_header_it_was_given() {
    assert_eq!(in_function("for{\n}"), ["for {", "}"]);
    assert_eq!(in_function("for  a<b  {\n}"), ["for a < b {", "}"]);
    assert_eq!(in_function("for  a  in  b  {\n}"), ["for a in b {", "}"]);
}

#[test]
fn an_empty_block_still_spans_two_lines() {
    assert_eq!(in_function("for {\n}"), ["for {", "}"]);
}

#[test]
fn an_else_continues_the_line_of_the_brace_it_follows() {
    assert_eq!(
        in_function("if a {\nb\n} else if c {\nd\n} else {\ne\n}"),
        [
            "if a {",
            "    b",
            "} else if c {",
            "    d",
            "} else {",
            "    e",
            "}",
        ]
    );
}

#[test]
fn a_match_writes_one_arm_per_line() {
    assert_eq!(
        in_function("match p {\nPending =>\n\"w\"\n\nFailed(r) => r\n}"),
        [
            "match p {",
            "    Pending => \"w\"",
            "    Failed(r) => r",
            "}",
        ]
    );
}

#[test]
fn a_block_holds_no_blank_lines() {
    assert_eq!(
        formatted("fn f() {\n\n    a := 1\n\n    b := 2\n\n}\n"),
        "fn f() {\n    a := 1\n    b := 2\n}\n"
    );
}

#[test]
fn a_record_literal_in_a_header_is_parenthesised() {
    assert_eq!(
        in_function("if (user { active: true }).active {\n}"),
        ["if (user { active: true }).active {", "}"]
    );
}

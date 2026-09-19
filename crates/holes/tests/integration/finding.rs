//! The behaviours of `docs/specs/holes.md`: what counts as a hole, and where one is found.

use lumen_diagnostics::render;

use crate::common::{holes, written};

#[test]
fn a_module_with_nothing_unfinished_holds_no_holes() {
    let source = "fn count() -> Int {\n    7\n}\n";

    assert_eq!(holes(source).len(), 0);
}

#[test]
fn a_body_written_as_todo_is_a_hole() {
    let source = "fn count() -> Int {\n    todo(\"count them\")\n}\n";

    assert_eq!(written(source), vec!["todo(\"count them\")"]);
}

#[test]
fn a_hole_takes_whatever_type_the_place_it_is_written_in_expects() {
    let source = concat!(
        "fn named() -> String {\n    todo(\"a name\")\n}\n\n",
        "fn counted() -> Int {\n    todo(\"a count\")\n}\n\n",
        "fn held() -> Option<Int> {\n    todo(\"an answer\")\n}\n"
    );

    assert_eq!(holes(source).len(), 3);
}

#[test]
fn every_hole_is_found_in_the_order_it_is_written() {
    let source = concat!(
        "fn go(count: Int) -> Int {\n",
        "    if count > 1 {\n        return todo(\"the first\")\n    }\n",
        "    total := todo(\"the second\")\n",
        "    total + todo(\"the third\")\n}\n"
    );

    assert_eq!(
        written(source),
        vec![
            "todo(\"the first\")",
            "todo(\"the second\")",
            "todo(\"the third\")"
        ]
    );
}

#[test]
fn a_hole_inside_a_loop_is_found_like_any_other() {
    let source = concat!(
        "fn go(counts: List<Int>) -> Int {\n",
        "    var total = 0\n",
        "    for count in counts {\n        total += todo(\"weigh it\")\n    }\n",
        "    total\n}\n"
    );

    assert_eq!(written(source), vec!["todo(\"weigh it\")"]);
}

#[test]
fn a_hole_inside_a_match_arm_is_found_like_any_other() {
    let source = concat!(
        "fn go(held: Option<Int>) -> Int {\n",
        "    match held {\n        Some(value) => value\n        None => todo(\"decide\")\n    }\n}\n"
    );

    assert_eq!(written(source), vec!["todo(\"decide\")"]);
}

#[test]
fn a_hole_written_inside_a_hole_is_two_holes() {
    let source = "fn named() -> String {\n    todo(todo(\"the reason itself\"))\n}\n";

    assert_eq!(
        written(source),
        vec![
            "todo(todo(\"the reason itself\"))",
            "todo(\"the reason itself\")"
        ]
    );
}

/// Nothing hides `todo`, so a hole is a hole: `docs/specs/modules.md` is what makes that true.
#[test]
fn the_hole_is_the_only_thing_that_may_be_named_todo() {
    let source = concat!(
        "fn count() -> Int {\n    todo(\"not a hole at all\")\n}\n\n",
        "fn todo(reason: String) -> Int {\n    7\n}\n"
    );

    let program = lumen_parser::parse(source).expect("the module parses");

    let refusal = lumen_resolver::resolve(program).expect_err("`todo` is already in scope");
    assert_eq!(refusal.message(), "`todo` is already in scope here");
}

#[test]
fn a_hole_is_refused_with_the_code_and_the_words_a_build_uses() {
    let source = "fn count() -> Int {\n    todo(\"count them\")\n}\n";
    let found = holes(source);
    let hole = found.first().expect("the module holds a hole");

    let shown = render(&hole.diagnostic(), source, "demo.lm");

    assert!(
        shown.starts_with("error[L0600]: this hole is not compiled\n"),
        "{shown}"
    );
}

#[test]
fn a_refused_hole_shows_the_line_the_reason_is_written_on() {
    let source = "fn count() -> Int {\n    todo(\"count them once the walk is written\")\n}\n";
    let found = holes(source);
    let hole = found.first().expect("the module holds a hole");

    let shown = render(&hole.diagnostic(), source, "demo.lm");

    assert!(
        shown.contains("count them once the walk is written"),
        "the reason is written where the hole is, so the rendered line carries it:\n{shown}"
    );
}

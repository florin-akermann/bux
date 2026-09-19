//! The invariants of `docs/specs/holes.md`, checked on generated modules.

use hegel::TestCase;
use hegel::generators as gs;

use crate::common::{holes, written};

/// The types a hole is written in, which between them cover every shape a type has.
const RESULTS: [&str; 5] = ["Int", "String", "Bool", "Option<Int>", "List<Int>"];

/// The places a hole is written, each holding it where `{}` is.
const PLACES: [&str; 4] = [
    "fn go() -> {result} {\n    {}\n}\n",
    "fn go() -> {result} {\n    held := {}\n    held\n}\n",
    "fn go(flag: Bool) -> {result} {\n    if flag {\n        return {}\n    }\n    {}\n}\n",
    "fn go(flag: Bool) -> {result} {\n    match flag {\n        true => {}\n        false => {}\n    }\n}\n",
];

/// The modules a property is checked over, none of which holds a hole.
const FINISHED: [&str; 4] = [
    "fn answer() -> Int {\n    7\n}\n",
    "fn told(word: String) -> String {\n    word + \"!\"\n}\n",
    "fn walked(counts: List<Int>) -> Int {\n    var total = 0\n    for count in counts {\n        total += count\n    }\n    total\n}\n",
    "fn held(maybe: Option<Int>) -> Int {\n    or(maybe, 0)\n}\n",
];

#[hegel::test]
fn a_hole_is_a_hole_whatever_type_it_is_written_in(tc: TestCase) {
    let result = tc.draw(gs::sampled_from(&RESULTS));
    let source = PLACES[0]
        .replace("{result}", result)
        .replace("{}", "todo(\"why\")");

    assert_eq!(written(&source), vec!["todo(\"why\")"], "{source}");
}

#[hegel::test]
fn every_hole_a_module_holds_is_found_wherever_it_is_written(tc: TestCase) {
    let place = tc.draw(gs::sampled_from(&PLACES));
    let result = tc.draw(gs::sampled_from(&RESULTS));
    let source = place
        .replace("{result}", result)
        .replace("{}", "todo(\"why\")");

    assert_eq!(
        holes(&source).len(),
        place.matches("{}").count(),
        "{source}"
    );
}

#[hegel::test]
fn a_module_with_nothing_unfinished_holds_no_hole_to_refuse(tc: TestCase) {
    let source = tc.draw(gs::sampled_from(&FINISHED));

    assert_eq!(holes(source), Vec::new(), "{source}");
}

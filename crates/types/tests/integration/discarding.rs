//! `docs/specs/discarding.md`: a value nothing takes, and the way to throw one away on purpose.

use crate::common::{inferred, refusal};

/// A module whose `main` takes `takes`, gives back `gives`, and holds `body`.
///
/// `save` gives back a `Result` and `log` gives back nothing, so a body written from the two of
/// them says exactly what it leaves behind.
fn module(takes: &str, gives: &str, body: &str) -> String {
    format!(
        concat!(
            "fn main({takes}) -> {gives} {{\n{body}\n}}\n\n",
            "fn save(value: Int) -> Result<(), String> {{\n    Ok(())\n}}\n\n",
            "fn log(value: Int) -> () {{\n    ()\n}}\n"
        ),
        takes = takes,
        gives = gives,
        body = body
    )
}

/// What every refusal here says, naming the `Result` the statement left behind.
const LEFT_BEHIND: &str = "`Result<(), String>` is left here and nothing takes it";

/// A `main` that takes nothing and gives back nothing, which most of these are.
const NOTHING: &str = "()";

#[test]
fn a_statement_that_leaves_a_value_behind_names_the_type_it_leaves() {
    let error = refusal(&module("", NOTHING, "    save(1)\n    ()"));

    assert_eq!(error.message(), LEFT_BEHIND);
}

#[test]
fn the_refusal_says_how_to_throw_the_value_away_on_purpose() {
    let error = refusal(&module("", NOTHING, "    save(1)\n    ()"));

    assert_eq!(
        error.help(),
        "write `_ = ` in front of it to throw the value away on purpose"
    );
}

#[test]
fn an_underscore_and_an_equals_make_the_same_statement_compile() {
    inferred(&module("", NOTHING, "    _ = save(1)\n    ()"));
}

#[test]
fn a_statement_of_unit_is_accepted_wherever_it_is_written() {
    inferred(&module("", NOTHING, "    log(1)\n    log(2)\n    ()"));
}

#[test]
fn the_last_statement_of_a_body_is_the_value_the_function_gives_back() {
    inferred(&module("", "Result<(), String>", "    save(1)"));
}

#[test]
fn the_last_statement_of_a_for_body_is_discarded_like_the_rest() {
    let walked = "    for value in values {\n        save(value)\n    }";

    let error = refusal(&module("values: List<Int>", NOTHING, walked));

    assert_eq!(error.message(), LEFT_BEHIND);
}

#[test]
fn a_for_body_that_ends_in_unit_is_accepted() {
    let walked = "    for value in values {\n        _ = save(value)\n    }";

    inferred(&module("values: List<Int>", NOTHING, walked));
}

#[test]
fn a_discarded_if_is_refused_once_rather_than_once_per_branch() {
    let branched =
        "    if count > 1 {\n        save(1)\n    } else {\n        save(2)\n    }\n    ()";

    let error = refusal(&module("count: Int", NOTHING, branched));

    assert_eq!(error.message(), LEFT_BEHIND);
}

#[test]
fn a_hole_written_as_a_statement_takes_the_unit_expected_of_it() {
    inferred(&module("", NOTHING, "    todo(\"not yet\")\n    ()"));
}

#[test]
fn a_discarded_comparison_is_named_as_the_bool_it_settles_to() {
    let error = refusal(&module("", NOTHING, "    1 == 2\n    ()"));

    assert_eq!(error.message(), "`Bool` is left here and nothing takes it");
}

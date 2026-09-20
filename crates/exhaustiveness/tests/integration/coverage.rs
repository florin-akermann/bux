//! What a `match` has to cover, and what it is told when it does not.
//!
//! `docs/specs/exhaustiveness.md` states each of these.

use lumen_diagnostics::render;

use lumen_exhaustiveness::MatchError;

use crate::common::{covers_everything, demo_payments, payments, refusal};

#[test]
fn a_match_on_every_variant_of_a_type_covers_it() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Pending => \"waiting\"\n",
        "        Authorized { authorization_id } => authorization_id\n",
        "        Failed(reason) => reason\n",
        "    }"
    ));

    covers_everything(&source);
}

#[test]
fn a_match_missing_a_variant_names_the_one_it_leaves_uncovered() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Pending => \"waiting\"\n",
        "        Authorized { authorization_id } => authorization_id\n",
        "    }"
    ));

    leaves_uncovered(&refusal(&source), "`Failed(_)`");
}

#[test]
fn a_match_missing_several_variants_names_them_in_the_order_they_are_declared() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Pending => \"waiting\"\n",
        "    }"
    ));

    leaves_uncovered(&refusal(&source), "`Authorized(_)`, `Failed(_)`");
}

#[test]
fn a_name_that_binds_covers_whatever_the_arms_before_it_did_not() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Pending => \"waiting\"\n",
        "        other => \"something else\"\n",
        "    }"
    ));

    covers_everything(&source);
}

#[test]
fn a_refusal_points_at_the_whole_match() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Pending => \"waiting\"\n",
        "    }"
    ));
    let start = source.find("match payment").expect("the match is written");

    let span = refusal(&source).span();

    assert_eq!(span.start(), start);
    assert!(
        source[span.start()..span.end()].ends_with("\"waiting\"\n    }"),
        "{:?} is the whole match",
        &source[span.start()..span.end()]
    );
}

#[test]
fn a_match_on_a_record_type_is_covered_by_its_one_constructor() {
    let source = concat!(
        "fn named(user: User) -> String {\n    match user {\n        User { name } => name\n    }\n}\n\n",
        "type User = {\n    name: String\n}\n"
    );

    covers_everything(source);
}

#[test]
fn a_match_on_a_bool_needs_both_of_its_values() {
    let source = concat!(
        "fn describe(count: Int) -> String {\n",
        "    match count > 1 {\n        true => \"yes\"\n    }\n}\n"
    );

    leaves_uncovered(&refusal(source), "`false`");
}

#[test]
fn a_match_on_a_bool_that_writes_both_of_its_values_covers_it() {
    let source = concat!(
        "fn describe(count: Int) -> String {\n",
        "    match count > 1 {\n        true => \"yes\"\n        false => \"no\"\n    }\n}\n"
    );

    covers_everything(source);
}

#[test]
fn a_number_has_more_values_than_a_match_can_write_down() {
    let source = concat!(
        "fn describe(count: Int) -> String {\n",
        "    match count {\n        0 => \"none\"\n        1 => \"one\"\n    }\n}\n"
    );

    leaves_uncovered(&refusal(source), "`_`");
}

#[test]
fn a_number_is_covered_by_a_name_that_binds() {
    let source = concat!(
        "fn describe(count: Int) -> String {\n",
        "    match count {\n        0 => \"none\"\n        other => \"some\"\n    }\n}\n"
    );

    covers_everything(source);
}

#[test]
fn a_string_has_more_values_than_a_match_can_write_down() {
    let source = concat!(
        "fn describe(word: String) -> String {\n",
        "    match word {\n        \"yes\" => \"agreed\"\n    }\n}\n"
    );

    leaves_uncovered(&refusal(source), "`_`");
}

#[test]
fn an_option_is_covered_by_its_two_prelude_variants() {
    let source = concat!(
        "fn or_else(value: Option<Int>, fallback: Int) -> Int {\n",
        "    match value {\n        Some(inner) => inner\n        None => fallback\n    }\n}\n"
    );

    covers_everything(source);
}

#[test]
fn an_option_missing_one_of_its_variants_is_refused() {
    let source = concat!(
        "fn unwrap(value: Option<Int>) -> Int {\n",
        "    match value {\n        Some(inner) => inner\n    }\n}\n"
    );

    leaves_uncovered(&refusal(source), "`None`");
}

#[test]
fn a_result_is_covered_by_its_two_prelude_variants() {
    let source = concat!(
        "fn describe(outcome: Result<Int, String>) -> String {\n",
        "    match outcome {\n        Ok(value) => \"ok\"\n        Err(problem) => problem\n    }\n}\n"
    );

    covers_everything(source);
}

#[test]
fn the_arms_that_reach_inside_a_nested_value_cover_it_between_them() {
    let source = concat!(
        "fn describe(outcome: Option<Result<Int, String>>) -> String {\n",
        "    match outcome {\n",
        "        Some(Ok(value)) => \"ok\"\n",
        "        Some(Err(problem)) => problem\n",
        "        None => \"nothing\"\n",
        "    }\n}\n"
    );

    covers_everything(source);
}

#[test]
fn a_nested_value_left_uncovered_is_named_where_it_is_uncovered() {
    let source = concat!(
        "fn describe(outcome: Option<Result<Int, String>>) -> String {\n",
        "    match outcome {\n",
        "        Some(Ok(value)) => \"ok\"\n",
        "        None => \"nothing\"\n",
        "    }\n}\n"
    );

    leaves_uncovered(&refusal(source), "`Some(Err(_))`");
}

#[test]
fn a_match_written_inside_another_match_is_checked_too() {
    let source = concat!(
        "fn describe(outcome: Option<Bool>) -> String {\n",
        "    match outcome {\n",
        "        Some(flag) => match flag {\n            true => \"yes\"\n        }\n",
        "        None => \"nothing\"\n",
        "    }\n}\n"
    );

    leaves_uncovered(&refusal(source), "`false`");
}

#[test]
fn a_match_written_in_a_loop_is_checked_too() {
    let source = concat!(
        "fn describe(flags: List<Bool>) -> Int {\n",
        "    for flag in flags {\n",
        "        _ = match flag {\n            true => 1\n        }\n",
        "    }\n    0\n}\n"
    );

    leaves_uncovered(&refusal(source), "`false`");
}

#[test]
fn the_first_match_that_leaves_a_value_uncovered_is_the_one_reported() {
    let source = concat!(
        "fn describe(first: Int, second: Int) -> Int {\n",
        "    earlier := match first > 1 {\n        true => 1\n    }\n",
        "    later := match second > 1 {\n        false => 2\n    }\n",
        "    earlier + later\n}\n"
    );

    leaves_uncovered(&refusal(source), "`false`");
}

#[test]
fn a_refusal_reads_as_the_diagnostic_it_is() {
    let source = concat!(
        "fn describe(count: Int) -> String {\n",
        "    match count > 1 {\n        true => \"yes\"\n    }\n}\n"
    );

    assert_eq!(
        render(&refusal(source).diagnostic(), source, "demo.lm"),
        concat!(
            "error[L0500]: this `match` does not cover `false`\n",
            "  --> demo.lm:2:5\n",
            "\n",
            "  2 |     match count > 1 {\n",
            "    |     ^^^^^^^^^^^^^^^^^ this runs on to line 4\n",
            "\n",
            "help: every value has an arm, or a name that binds whatever the arms before it did not\n",
        )
    );
}

#[test]
fn a_value_left_uncovered_under_an_arm_is_named_alongside_the_variants_with_no_arm() {
    let source = concat!(
        "fn describe(held: Held) -> Int {\n    match held {\n        Wrapping(Some(value)) => value\n    }\n}\n\n",
        "type Held =\n    | Wrapping(Option<Int>)\n    | Empty\n"
    );

    leaves_uncovered(&refusal(source), "`Wrapping(None)`, `Empty`");
}

#[test]
fn a_bool_left_uncovered_under_an_arm_is_named_where_it_is_uncovered() {
    let source = concat!(
        "fn describe(flag: Option<Bool>) -> Int {\n",
        "    match flag {\n        Some(true) => 1\n        None => 0\n    }\n}\n"
    );

    leaves_uncovered(&refusal(source), "`Some(false)`");
}

#[test]
fn a_constructor_carrying_more_than_one_value_names_a_stand_in_for_each() {
    let source = concat!(
        "fn first(pair: Pair) -> Int {\n    match pair {\n        Neither => 0\n    }\n}\n\n",
        "type Pair =\n    | Both(Int, Int)\n    | Neither\n"
    );

    leaves_uncovered(&refusal(source), "`Both(_, _)`");
}

#[test]
fn a_match_over_a_type_of_another_module_covers_the_variants_that_module_declares() {
    let module = demo_payments(concat!(
        "        demo.Pending => \"pending\"\n",
        "        demo.Authorized { authorization_id } => authorization_id\n",
        "        demo.Failed(reason) => reason\n"
    ));

    module.covers_everything();
}

#[test]
fn a_match_over_a_type_of_another_module_names_the_variant_it_leaves_out() {
    let module = demo_payments(concat!(
        "        demo.Pending => \"pending\"\n",
        "        demo.Failed(reason) => reason\n"
    ));

    leaves_uncovered(&module.refusal(), "`demo.Authorized(_)`");
}

#[test]
fn an_underscore_covers_every_value_the_arms_above_it_left() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Pending => \"waiting\"\n",
        "        _ => \"the rest\"\n",
        "    }"
    ));

    covers_everything(&source);
}

#[test]
fn an_or_pattern_covers_what_its_alternatives_cover_between_them() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Pending | Failed(_) => \"seen\"\n",
        "        Authorized { authorization_id } => authorization_id\n",
        "    }"
    ));

    covers_everything(&source);
}

#[test]
fn an_or_pattern_covering_all_but_one_variant_names_the_one_it_leaves() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Pending | Failed(_) => \"seen\"\n",
        "    }"
    ));

    leaves_uncovered(&refusal(&source), "`Authorized(_)`");
}

#[test]
fn a_whole_number_pattern_leaves_every_other_number_uncovered() {
    let source =
        "fn counted(count: Int) -> String {\n    match count {\n        0 => \"none\"\n    }\n}\n";

    leaves_uncovered(&refusal(source), "`_`");
}

#[test]
fn a_whole_number_pattern_is_covered_by_an_underscore_after_it() {
    let source = "fn counted(count: Int) -> String {\n    match count {\n        0 => \"none\"\n        _ => \"some\"\n    }\n}\n";

    covers_everything(source);
}

#[test]
fn an_or_pattern_of_numbers_still_leaves_the_rest_of_them_uncovered() {
    let source = "fn counted(count: Int) -> String {\n    match count {\n        0 | 1 => \"few\"\n    }\n}\n";

    leaves_uncovered(&refusal(source), "`_`");
}

#[test]
fn an_alternative_inside_a_constructor_covers_only_what_it_names() {
    let source = concat!(
        "fn described(held: Option<Bool>) -> String {\n    match held {\n",
        "        Some(true | false) => \"held\"\n",
        "        None => \"nothing\"\n    }\n}\n"
    );

    covers_everything(source);
}

/// Asserts that `refused` names `uncovered` as what the `match` it refuses leaves unanswered.
fn leaves_uncovered(refused: &MatchError, uncovered: &str) {
    assert_eq!(
        refused.message(),
        format!("this `match` does not cover {uncovered}")
    );
}

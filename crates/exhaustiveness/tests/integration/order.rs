//! The order a `match` lists its arms in, which is the order the type declares its variants.
//!
//! `docs/specs/exhaustiveness.md` states the rule, and `docs/design.md` section 13 is where it
//! comes from: a new variant has one place to be handled, and no diff is ever reorder-only.

use lumen_diagnostics::render;

use lumen_exhaustiveness::MatchError;

use crate::common::{covers_everything, demo_payments, outcomes, payments, refusal};

#[test]
fn a_match_whose_arms_are_in_the_order_the_type_declares_them_passes() {
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
fn an_arm_written_above_one_the_type_declares_above_it_is_refused() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Failed(reason) => reason\n",
        "        Pending => \"waiting\"\n",
        "        Authorized { authorization_id } => authorization_id\n",
        "    }"
    ));

    writes_out_of_order(&refusal(&source), "`Pending` after `Failed`");
}

#[test]
fn a_refusal_points_at_the_arm_that_is_out_of_place() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Failed(reason) => reason\n",
        "        Pending => \"waiting\"\n",
        "        Authorized { authorization_id } => authorization_id\n",
        "    }"
    ));
    let at = source.find("Pending =>").expect("the arm is written");

    let span = refusal(&source).span();

    assert_eq!(span.start(), at);
    assert_eq!(&source[span.start()..span.end()], "Pending");
}

#[test]
fn a_name_that_binds_is_placed_by_nothing_and_may_be_written_anywhere() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Authorized { authorization_id } => authorization_id\n",
        "        other => \"something else\"\n",
        "    }"
    ));

    covers_everything(&source);
}

#[test]
fn a_literal_arm_is_placed_by_nothing_because_no_declaration_writes_its_values_down() {
    let source = concat!(
        "fn counted(count: Int) -> String {\n",
        "    match count {\n        1 => \"one\"\n        0 => \"none\"\n        other => \"some\"\n    }\n}\n"
    );

    covers_everything(source);
}

#[test]
fn the_two_values_of_a_bool_are_written_either_way() {
    let source = concat!(
        "fn spoken(count: Int) -> String {\n",
        "    match count > 1 {\n        true => \"yes\"\n        false => \"no\"\n    }\n}\n"
    );

    covers_everything(source);
}

#[test]
fn an_option_is_written_the_way_the_prelude_declares_it() {
    let source = concat!(
        "fn or_else(value: Option<Int>, fallback: Int) -> Int {\n",
        "    match value {\n        None => fallback\n        Some(inner) => inner\n    }\n}\n"
    );

    writes_out_of_order(&refusal(source), "`Some` after `None`");
}

#[test]
fn two_arms_that_reach_inside_one_variant_are_in_order_with_each_other() {
    let source = outcomes(concat!(
        "        Some(Ok(value)) => \"ok\"\n",
        "        Some(Err(problem)) => problem\n",
        "        None => \"nothing\"\n"
    ));

    covers_everything(&source);
}

#[test]
fn two_arms_that_reach_inside_one_variant_out_of_order_are_refused() {
    let source = outcomes(concat!(
        "        Some(Err(problem)) => problem\n",
        "        Some(Ok(value)) => \"ok\"\n",
        "        None => \"nothing\"\n"
    ));

    writes_out_of_order(&refusal(&source), "`Ok` after `Err`");
}

#[test]
fn a_refusal_from_inside_a_variant_points_at_the_whole_arm_that_is_out_of_place() {
    let source = outcomes(concat!(
        "        Some(Err(problem)) => problem\n",
        "        Some(Ok(value)) => \"ok\"\n",
        "        None => \"nothing\"\n"
    ));
    let at = source.find("Some(Ok(value))").expect("the arm is written");

    let span = refusal(&source).span();

    assert_eq!(span.start(), at);
    assert_eq!(&source[span.start()..span.end()], "Some(Ok(value))");
}

#[test]
fn the_variant_an_arm_answers_for_places_it_before_what_it_reaches_for_does() {
    let source = outcomes(concat!(
        "        None => \"nothing\"\n",
        "        Some(Ok(value)) => \"ok\"\n",
        "        Some(Err(problem)) => problem\n"
    ));

    writes_out_of_order(&refusal(&source), "`Some` after `None`");
}

#[test]
fn a_name_that_binds_inside_a_variant_stops_the_arm_being_placed_any_deeper() {
    let source = outcomes(concat!(
        "        Some(Err(problem)) => problem\n",
        "        Some(inner) => \"something\"\n",
        "        None => \"nothing\"\n"
    ));

    covers_everything(&source);
}

#[test]
fn the_two_values_of_a_bool_inside_a_variant_are_written_either_way() {
    let source = concat!(
        "fn described(flag: Option<Bool>) -> Int {\n",
        "    match flag {\n",
        "        Some(true) => 1\n",
        "        Some(false) => 0\n",
        "        None => 2\n",
        "    }\n}\n"
    );

    covers_everything(source);
}

#[test]
fn a_refusal_reads_as_the_diagnostic_it_is() {
    let source = concat!(
        "fn or_else(value: Option<Int>, fallback: Int) -> Int {\n",
        "    match value {\n        None => fallback\n        Some(inner) => inner\n    }\n}\n"
    );

    assert_eq!(
        render(&refusal(source).diagnostic(), source, "demo.lm"),
        concat!(
            "error[L0501]: this `match` writes `Some` after `None`\n",
            "  --> demo.lm:4:9\n",
            "\n",
            "  4 |         Some(inner) => inner\n",
            "    |         ^^^^^^^^^^^\n",
            "\n",
            "help: arms come in the order the type declares its variants: `Some` before `None`\n",
        )
    );
}

#[test]
fn a_match_over_a_type_of_another_module_lists_its_arms_in_that_module_s_order() {
    let module = demo_payments(concat!(
        "        demo.Failed(reason) => reason\n",
        "        demo.Pending => \"pending\"\n",
        "        demo.Authorized { authorization_id } => authorization_id\n"
    ));

    writes_out_of_order(&module.refusal(), "`demo.Pending` after `demo.Failed`");
}

/// The refusal points at the alternative rather than the whole arm, which is what is out of place.
#[test]
fn an_alternative_written_above_one_the_type_declares_above_it_is_refused() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Failed(_) | Pending => \"seen\"\n",
        "        Authorized { authorization_id } => authorization_id\n",
        "    }"
    ));

    let refused = refusal(&source);

    writes_out_of_order(&refused, "`Pending` after `Failed`");
    let span = refused.span();
    assert_eq!(&source[span.start()..span.end()], "Pending");
}

#[test]
fn an_arm_is_placed_by_the_first_alternative_it_writes() {
    let source = payments(concat!(
        "    match payment {\n",
        "        Pending | Failed(_) => \"seen\"\n",
        "        Authorized { authorization_id } => authorization_id\n",
        "    }"
    ));

    covers_everything(&source);
}

#[test]
fn an_alternative_inside_a_constructor_is_held_to_the_order_as_well() {
    let source = concat!(
        "fn described(held: Option<Payment>) -> String {\n    match held {\n",
        "        Some(Failed(_) | Pending) => \"seen\"\n",
        "        Some(Authorized { authorization_id }) => authorization_id\n",
        "        None => \"nothing\"\n    }\n}\n\n",
        "type Payment =\n    | Pending\n    | Authorized {\n        authorization_id: String\n    }\n    | Failed(String)\n"
    );

    writes_out_of_order(&refusal(source), "`Pending` after `Failed`");
}

/// Asserts that `refused` names the arm out of place and the one the type declares below it.
fn writes_out_of_order(refused: &MatchError, written: &str) {
    assert_eq!(refused.message(), format!("this `match` writes {written}"));
}

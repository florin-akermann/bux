//! `docs/specs/calls.md`: the two spellings of one call lower to the same instructions.

use crate::common::{body_of, lowered};

/// A module calling `held` the way `call` writes it, and declaring what it calls below.
fn calling_held(call: &str) -> String {
    format!(
        "fn go() -> String {{\n    count := 1\n    {call}\n}}\n\n\
         fn held(count: Int, said: String) -> String {{\n    said\n}}\n\n\
         fn twice(count: Int) -> Int {{\n    count + count\n}}\n"
    )
}

#[test]
fn a_call_with_its_first_argument_in_front_is_the_instructions_the_plain_call_is() {
    let in_front = lowered(&calling_held("count.held(\"hi\")"));
    let plainly = lowered(&calling_held("held(count, \"hi\")"));

    assert_eq!(
        body_of(&in_front, "go").instructions,
        body_of(&plainly, "go").instructions
    );
}

#[test]
fn anything_that_is_a_value_stands_before_the_dot_and_is_passed_first() {
    let in_front = lowered(&calling_held("twice(count).held(\"hi\")"));
    let plainly = lowered(&calling_held("held(twice(count), \"hi\")"));

    assert_eq!(
        body_of(&in_front, "go").instructions,
        body_of(&plainly, "go").instructions
    );
}

#[test]
fn a_trait_method_in_front_form_reaches_the_instance_the_plain_call_reaches() {
    let source = |call: &str| {
        format!(
            "derive Show for Payment\n\n\
             fn said(payment: Payment) -> String {{\n    {call}\n}}\n\n\
             type Payment = Payment(Int)\n"
        )
    };
    let in_front = lowered(&source("payment.shown()"));
    let plainly = lowered(&source("shown(payment)"));

    assert_eq!(
        body_of(&in_front, "said").instructions,
        body_of(&plainly, "said").instructions
    );
}

#[test]
fn a_constructor_written_in_front_form_builds_what_the_plain_call_builds() {
    let source = |call: &str| {
        format!(
            "fn made(count: Int) -> Payment {{\n    {call}\n}}\n\n\
             type Payment = Payment(Int)\n"
        )
    };
    let in_front = lowered(&source("count.Payment()"));
    let plainly = lowered(&source("Payment(count)"));

    assert_eq!(
        body_of(&in_front, "made").instructions,
        body_of(&plainly, "made").instructions
    );
}

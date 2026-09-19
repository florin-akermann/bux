//! The invariants of `docs/specs/exhaustiveness.md`, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_exhaustiveness::check;
use lumen_parser::parse;
use lumen_resolver::resolve;

use crate::common::{covers_everything, payments, typed};

/// The arms a `match` on the payment type of `docs/design.md` may write, one per variant.
const ARMS: [&str; 3] = [
    "        Pending => \"waiting\"\n",
    "        Authorized { authorization_id } => authorization_id\n",
    "        Failed(reason) => reason\n",
];

#[hegel::test]
fn checking_never_panics_and_is_deterministic(tc: TestCase) {
    let source = tc.draw(gs::text());
    let Ok(program) = parse(&source) else {
        return;
    };
    let Ok(resolved) = resolve(program) else {
        return;
    };
    let Ok(inferred) = lumen_types::check(resolved) else {
        return;
    };
    assert_eq!(check(&inferred).is_ok(), check(&inferred).is_ok());
}

#[hegel::test]
fn a_match_on_every_variant_covers_whatever_order_the_arms_are_written_in(tc: TestCase) {
    let mut arms: Vec<&str> = ARMS.to_vec();
    let order = tc.draw(gs::sampled_from(&[0_usize, 1, 2]));
    arms.rotate_left(order);

    covers_everything(&payments(&matching(&arms)));
}

#[hegel::test]
fn a_match_with_one_of_its_variants_left_out_is_refused(tc: TestCase) {
    let left_out = tc.draw(gs::sampled_from(&[0_usize, 1, 2]));
    let arms: Vec<&str> = ARMS
        .iter()
        .enumerate()
        .filter(|(at, _)| *at != left_out)
        .map(|(_, arm)| *arm)
        .collect();

    let source = payments(&matching(&arms));

    check(&typed(&source)).expect_err(&format!("{source:?} is refused"));
}

#[hegel::test]
fn a_name_that_binds_covers_whatever_else_is_written(tc: TestCase) {
    let written = tc.draw(gs::sampled_from(&[0_usize, 1, 2, 3]));
    let mut arms: Vec<&str> = ARMS.iter().take(written).copied().collect();
    arms.push("        other => \"something else\"\n");

    covers_everything(&payments(&matching(&arms)));
}

#[hegel::test]
fn a_refusal_points_inside_the_source(tc: TestCase) {
    let left_out = tc.draw(gs::sampled_from(&[0_usize, 1, 2]));
    let arms: Vec<&str> = ARMS
        .iter()
        .enumerate()
        .filter(|(at, _)| *at != left_out)
        .map(|(_, arm)| *arm)
        .collect();
    let source = payments(&matching(&arms));

    let span = check(&typed(&source))
        .expect_err("a match missing a variant is refused")
        .span();

    assert!(span.start() < span.end(), "{span:?} is empty");
    assert!(span.end() <= source.len(), "{span:?} runs past the input");
}

/// A `match` on the payment the enclosing function takes, written with `arms`.
fn matching(arms: &[&str]) -> String {
    format!("    match payment {{\n{}    }}", arms.concat())
}

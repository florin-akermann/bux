//! The invariants of `docs/specs/exhaustiveness.md`, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_exhaustiveness::check;
use lumen_parser::parse;
use lumen_resolver::resolve;

use crate::common::{covers_everything, payments, typed};

/// The name the module under test is compiled as, which nothing here depends on.
const MODULE: &str = "demo";

/// The arms a `match` on the payment type of `docs/design.md` may write, one per variant.
const ARMS: [&str; 3] = [
    "        Pending => \"waiting\"\n",
    "        Authorized { authorization_id } => authorization_id\n",
    "        Failed(reason) => reason\n",
];

/// A type whose variants each carry nothing, so any arm of it may be an alternative.
const COLOURS: [&str; 3] = ["Red", "Green", "Blue"];

#[hegel::test]
fn checking_never_panics_and_is_deterministic(tc: TestCase) {
    let source = tc.draw(gs::text());
    let Ok(program) = parse(&source) else {
        return;
    };
    let Ok(resolved) = resolve(program, MODULE) else {
        return;
    };
    let Ok(inferred) = lumen_types::check(resolved, &lumen_types::Imported::default()) else {
        return;
    };
    assert_eq!(check(&inferred).is_ok(), check(&inferred).is_ok());
}

/// Every other order is refused, which is what makes a new variant have one place to be handled.
#[hegel::test]
fn a_match_whose_arms_are_rotated_out_of_the_declared_order_is_refused(tc: TestCase) {
    let rotation = tc.draw(gs::sampled_from(&[1_usize, 2]));
    let mut arms: Vec<&str> = ARMS.to_vec();
    arms.rotate_left(rotation);
    let source = payments(&matching(&arms));

    let refusal = check(&typed(&source)).expect_err(&format!("{source:?} is refused"));

    assert!(
        refusal.message().starts_with("this `match` writes "),
        "{}",
        refusal.message()
    );
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

/// `A | B` in one arm covers exactly what `A` and `B` cover in two arms.
#[hegel::test]
fn one_arm_of_alternatives_covers_what_the_same_arms_cover_separately(tc: TestCase) {
    let written = tc.draw(gs::sampled_from(&[1_usize, 2, 3]));
    let named = &COLOURS[..written];
    let apart = named
        .iter()
        .map(|name| format!("        {name} => \"seen\"\n"))
        .collect::<Vec<String>>()
        .concat();
    let together = format!("        {} => \"seen\"\n", named.join(" | "));

    assert_eq!(covers_every_colour(&apart), covers_every_colour(&together));
}

/// Whether a `match` on a colour whose arms are `arms` covers every colour there is.
fn covers_every_colour(arms: &str) -> bool {
    let source = format!(
        "fn described(colour: Colour) -> String {{\n    match colour {{\n{arms}    }}\n}}\n\n\
         type Colour =\n    | Red\n    | Green\n    | Blue\n"
    );
    check(&typed(&source)).is_ok()
}

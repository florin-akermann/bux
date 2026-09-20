//! `docs/specs/patterns.md`: `_` binds nothing, and neither does any alternative of an or-pattern.

use lumen_resolver::DefinitionKind;
use lumen_resolver::Namespace::Value;

use crate::common::{meaning, refusal, resolved};

/// A module declaring the payment type, with `arms` as the arms of the one `match`.
fn matching(arms: &str) -> String {
    format!(
        "fn describe(payment: Payment) -> String {{\n    match payment {{\n{arms}    }}\n}}\n\n\
         type Payment =\n    | Pending\n    | Running\n    | Failed(String)\n"
    )
}

#[test]
fn an_underscore_declares_no_name_for_the_arm_to_look_up() {
    let source = matching("        _ => \"whatever\"\n");

    resolved(&source);
}

#[test]
fn an_alternative_naming_a_variant_means_the_declaration_it_names() {
    let source = matching("        Pending | Running => \"waiting\"\n        _ => \"other\"\n");

    let named = meaning(&source, Value, "Pending", 1).expect("the alternative names the variant");
    assert_eq!(named.kind, DefinitionKind::Constructor);
}

#[test]
fn a_name_that_binds_inside_an_or_pattern_is_refused_where_it_is_written() {
    let source = matching("        Pending | anything => \"waiting\"\n");
    let refused = refusal(&source);

    assert_eq!(
        refused.message(),
        "`anything` binds inside an or-pattern, which binds nothing"
    );
    assert_eq!(refused.span().text(&source), "anything");
}

#[test]
fn the_help_of_a_binding_inside_an_or_pattern_says_to_write_one_arm_for_each() {
    let source = matching("        Pending | anything => \"waiting\"\n");

    assert_eq!(
        refusal(&source).help(),
        "write one arm for each alternative where one of them binds"
    );
}

#[test]
fn a_name_that_binds_inside_a_constructor_of_an_alternative_is_refused_as_well() {
    let source = matching("        Pending | Failed(reason) => \"waiting\"\n");

    assert_eq!(
        refusal(&source).message(),
        "`reason` binds inside an or-pattern, which binds nothing"
    );
}

#[test]
fn a_record_pattern_is_never_an_alternative_because_it_binds_every_field_it_names() {
    let source = concat!(
        "fn described(outcome: Outcome) -> String {\n    match outcome {\n",
        "        Ready | Held { held } => \"there\"\n    }\n}\n\n",
        "type Outcome =\n    | Ready\n    | Held {\n        held: String\n    }\n"
    );

    assert_eq!(
        refusal(source).message(),
        "`held` binds inside an or-pattern, which binds nothing"
    );
}

#[test]
fn a_name_that_binds_outside_an_or_pattern_binds_as_it_always_did() {
    let source = matching("        anything => \"whatever\"\n");

    let bound = meaning(&source, Value, "anything", 1).expect("the name binds the value");
    assert_eq!(bound.kind, DefinitionKind::Local);
}

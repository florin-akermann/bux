//! The printed form `docs/specs/modules.md` states, which the harness of the Bux resolver compares.

use crate::printed::{render, render_prelude};

#[test]
fn a_resolved_module_prints_each_name_in_the_order_of_its_span() {
    let source = "fn f(id: Id) -> Id {\n    id\n}\n\ntype Id = {\n    value: Int\n}\n";

    assert_eq!(
        render(source),
        concat!(
            "resolved\n",
            "3..4 value function declared 3..4\n",
            "5..7 value parameter declared 5..7\n",
            "9..11 type type declared 36..38\n",
            "16..18 type type declared 36..38\n",
            "25..27 value parameter declared 5..7\n",
            "36..38 type type declared 36..38\n",
            "36..38 value constructor declared 36..38\n",
            "54..57 type type prelude\n",
        )
    );
}

#[test]
fn a_refusal_prints_its_code_span_message_and_help() {
    assert_eq!(
        render("fn f() -> Int {\n    missing\n}\n"),
        concat!(
            "refused\n",
            "L0300 20..27 there is nothing named `missing`\n",
            "help: a name is declared in this file, imported, or supplied by the prelude\n",
        )
    );
}

#[test]
fn a_source_that_does_not_parse_prints_unparsed() {
    assert_eq!(render("fn ("), "unparsed\n");
}

#[test]
fn the_prelude_prints_as_a_resolved_module() {
    assert!(render_prelude().starts_with("resolved\n"));
}

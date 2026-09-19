//! Where a declaration belongs, which is above what it uses.
//!
//! `docs/specs/modules.md` states the rule and `docs/design.md` section 13 is where it comes
//! from: a file reads top down, so the reader meets the intent before the detail.

use lumen_diagnostics::render;

use crate::common::{refusal, resolved};

#[test]
fn a_declaration_written_above_what_uses_it_is_refused() {
    let error = refusal("fn one() -> Int {\n    1\n}\n\nfn two() -> Int {\n    one()\n}\n");

    assert_eq!(
        error.message(),
        "`one` is written above `two`, which uses it"
    );
}

#[test]
fn a_declaration_written_below_what_uses_it_passes() {
    let source = "fn two() -> Int {\n    one()\n}\n\nfn one() -> Int {\n    1\n}\n";

    assert_eq!(resolved(source).program().items.len(), 2);
}

#[test]
fn a_type_written_above_the_function_it_is_declared_for_is_refused() {
    let error =
        refusal("type UserId = UserId(Int)\n\nfn wrap(raw: Int) -> UserId {\n    UserId(raw)\n}\n");

    assert_eq!(
        error.message(),
        "`UserId` is written above `wrap`, which uses it"
    );
}

#[test]
fn a_type_written_above_the_type_that_holds_it_is_refused() {
    let error = refusal("type UserId = UserId(Int)\n\ntype User = {\n    id: UserId\n}\n");

    assert_eq!(
        error.message(),
        "`UserId` is written above `User`, which uses it"
    );
}

#[test]
fn two_declarations_that_use_each_other_are_written_either_way() {
    let source = concat!(
        "fn odd(count: Int) -> Bool {\n    even(count - 1)\n}\n\n",
        "fn even(count: Int) -> Bool {\n    odd(count - 1)\n}\n"
    );

    assert_eq!(resolved(source).program().items.len(), 2);
}

#[test]
fn a_declaration_a_cycle_reaches_only_one_way_round_is_still_placed() {
    let error = refusal(concat!(
        "fn helper() -> Int {\n    1\n}\n\n",
        "fn odd(count: Int) -> Bool {\n    even(count - helper())\n}\n\n",
        "fn even(count: Int) -> Bool {\n    odd(count - 1)\n}\n"
    ));

    assert_eq!(
        error.message(),
        "`helper` is written above `odd`, which uses it"
    );
}

#[test]
fn an_import_is_placed_by_the_formatter_and_not_by_what_uses_it() {
    let source = "import io\n\nfn greet() {\n    io.print(\"hi\")\n}\n";

    assert_eq!(resolved(source).program().items.len(), 2);
}

#[test]
fn a_refusal_reads_as_the_diagnostic_it_is() {
    let source = "fn one() -> Int {\n    1\n}\n\nfn two() -> Int {\n    one()\n}\n";

    assert_eq!(
        render(&refusal(source).diagnostic(), source, "demo.lm"),
        concat!(
            "error[L0303]: `one` is written above `two`, which uses it\n",
            "  --> demo.lm:1:4\n",
            "\n",
            "  1 | fn one() -> Int {\n",
            "    |    ^^^\n",
            "\n",
            "help: a file reads top down: move it below what uses it\n",
        )
    );
}

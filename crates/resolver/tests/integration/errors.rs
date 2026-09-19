//! The refusals of `docs/specs/modules.md`: one name has one definition, and every name has one.

use lumen_diagnostics::render;

use crate::common::refusal;

#[test]
fn a_name_with_no_definition_is_refused() {
    let error = refusal("fn f() -> Int {\n    missing\n}\n");

    assert_eq!(error.message(), "there is nothing named `missing`");
}

#[test]
fn a_type_that_is_not_declared_is_refused() {
    let error = refusal("fn f() -> Money {\n    1\n}\n");

    assert_eq!(error.message(), "there is no type named `Money`");
}

#[test]
fn a_module_may_not_declare_one_name_twice() {
    let error = refusal("fn f() -> Int {\n    1\n}\n\nfn f() -> Int {\n    2\n}\n");

    assert_eq!(error.message(), "`f` is declared twice in this module");
}

#[test]
fn two_parameters_of_one_function_may_not_share_a_name() {
    let error = refusal("fn f(a: Int, a: Int) -> Int {\n    1\n}\n");

    assert_eq!(error.message(), "`a` is declared twice in this module");
}

#[test]
fn a_binding_may_not_hide_a_parameter() {
    let error = refusal("fn f(total: Int) -> Int {\n    total := 1\n    total\n}\n");

    assert_eq!(error.message(), "`total` is already in scope here");
}

#[test]
fn a_declaration_may_not_hide_a_name_the_prelude_supplies() {
    let error = refusal("fn Ok() -> Int {\n    1\n}\n");

    assert_eq!(error.message(), "`Ok` is already in scope here");
}

#[test]
fn a_binding_does_not_see_the_name_it_is_binding() {
    let error = refusal("fn f() -> Int {\n    total := total + 1\n    total\n}\n");

    assert_eq!(error.message(), "there is nothing named `total`");
}

#[test]
fn a_loop_binding_is_gone_once_the_loop_ends() {
    let error = refusal(
        "fn f(users: List<Int>) -> Int {\n    for user in users {\n        user\n    }\n    user\n}\n",
    );

    assert_eq!(error.message(), "there is nothing named `user`");
}

#[test]
fn a_pattern_binding_is_gone_once_its_arm_ends() {
    let source = "type Payment =\n    | Pending\n    | Failed(String)\n\nfn describe(payment: Payment) -> String {\n    match payment {\n        Pending => reason\n        Failed(reason) => reason\n    }\n}\n";

    assert_eq!(refusal(source).message(), "there is nothing named `reason`");
}

#[test]
fn a_refusal_reads_as_the_diagnostic_it_is() {
    let source = "fn f() -> Int {\n    missing\n}\n";
    let error = refusal(source);

    assert_eq!(
        render(&error.diagnostic(), source, "demo.lm"),
        concat!(
            "error[L0300]: there is nothing named `missing`\n",
            "  --> demo.lm:2:5\n",
            "\n",
            "  2 |     missing\n",
            "    |     ^^^^^^^\n",
            "\n",
            "help: a name is declared in this file, imported, or supplied by the prelude\n",
        )
    );
}

#[test]
fn each_kind_of_refusal_carries_its_own_code() {
    assert_eq!(opening("fn f() -> Int {\n    missing\n}\n"), "error[L0300]");
    assert_eq!(
        opening("fn f() -> Int {\n    1\n}\n\nfn f() -> Int {\n    2\n}\n"),
        "error[L0301]"
    );
    assert_eq!(opening("fn Ok() -> Int {\n    1\n}\n"), "error[L0302]");
}

/// The `error[LNNNN]` that `source` is refused with.
fn opening(source: &str) -> String {
    let rendered = render(&refusal(source).diagnostic(), source, "demo.lm");
    rendered
        .split_once(']')
        .map(|(opening, _)| format!("{opening}]"))
        .expect("a rendering opens with its code")
}

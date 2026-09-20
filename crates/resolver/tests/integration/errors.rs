//! The refusals of `docs/specs/modules.md`: one name has one definition, and every name has one.

use lumen_diagnostics::render;

use crate::common::{refusal, resolved};

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
    let error = refusal("fn f() -> Int {\n    2\n}\n\nfn f() -> Int {\n    1\n}\n");

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
    let source = "fn describe(payment: Payment) -> String {\n    match payment {\n        Pending => reason\n        Failed(reason) => reason\n    }\n}\n\ntype Payment =\n    | Pending\n    | Failed(String)\n";

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
        opening("fn f() -> Int {\n    2\n}\n\nfn f() -> Int {\n    1\n}\n"),
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

#[test]
fn a_function_name_written_as_anything_but_a_call_is_refused() {
    let source = concat!(
        "fn go() -> Int {\n    held := helper\n    1\n}\n\n",
        "fn helper() -> Int {\n    1\n}\n"
    );
    let error = refusal(source);

    assert_eq!(
        error.message(),
        "`helper` is a function, so it is written as a call"
    );
    assert_eq!(
        error.help(),
        "version 0.1 reaches a function by calling it; write the call"
    );
    assert_eq!(error.span().text(source), "helper");
}

#[test]
fn a_function_name_is_refused_wherever_a_value_is_written() {
    let places = [
        "    return helper\n",
        "    helper + 1\n",
        "    take(helper)\n",
        "    match helper {\n        anything => 1\n    }\n",
    ];
    for place in places {
        let source = format!(
            concat!(
                "fn go() -> Int {{\n{place}}}\n\n",
                "fn take(value: Int) -> Int {{\n    value\n}}\n\n",
                "fn helper() -> Int {{\n    1\n}}\n"
            ),
            place = place
        );

        assert_eq!(
            refusal(&source).message(),
            "`helper` is a function, so it is written as a call",
            "{place} is refused"
        );
    }
}

#[test]
fn a_function_name_on_the_left_of_an_assignment_is_refused_like_any_other_value() {
    let source = concat!(
        "fn go() -> Int {\n    helper = 1\n    helper()\n}\n\n",
        "fn helper() -> Int {\n    1\n}\n"
    );

    assert_eq!(
        refusal(source).message(),
        "`helper` is a function, so it is written as a call"
    );
}

#[test]
fn a_module_name_written_as_a_value_is_refused() {
    let source = "import io\n\nfn go() -> Int {\n    held := io\n    1\n}\n";
    let error = refusal(source);

    assert_eq!(
        error.message(),
        "`io` is a module, so a name inside it is what is written"
    );
    assert_eq!(
        error.help(),
        "a module is what a name is reached through, as `io.println` is"
    );
}

#[test]
fn a_module_name_left_of_a_dot_is_resolved_rather_than_refused() {
    let source = "import io\n\nfn go() -> Int {\n    io.count()\n}\n";

    resolved(source);
}

#[test]
fn a_name_inside_a_module_is_written_as_a_call_because_nothing_holds_a_function() {
    let source = "import io\n\nfn go() -> Int {\n    io.count\n}\n";

    assert_eq!(
        refusal(source).message(),
        "`count` is a function, so it is written as a call"
    );
}

#[test]
fn a_function_called_by_its_name_is_resolved_rather_than_refused() {
    let source = concat!(
        "fn go() -> Int {\n    helper()\n}\n\n",
        "fn helper() -> Int {\n    1\n}\n"
    );

    resolved(source);
}

#[test]
fn an_immutable_binding_is_never_assigned_to() {
    let source = "fn count() -> Int {\n    total := 0\n    total = 2\n    total\n}\n";
    let error = refusal(source);

    assert_eq!(
        error.message(),
        "`total` is not a `var`, so it is never assigned to"
    );
    assert_eq!(
        error.help(),
        "mutation is explicit: bind it with `var`, or bind a new name"
    );
}

#[test]
fn a_var_binding_is_what_an_assignment_names() {
    let source = "fn count() -> Int {\n    var total = 0\n    total = 2\n    total\n}\n";

    resolved(source);
}

#[test]
fn a_parameter_is_a_binding_that_never_changes() {
    let source = "fn count(total: Int) -> Int {\n    total = 2\n    total\n}\n";

    assert_eq!(
        refusal(source).message(),
        "`total` is not a `var`, so it is never assigned to"
    );
}

#[test]
fn a_for_in_binding_is_a_binding_that_never_changes() {
    let source = "fn go(items: List<Int>) -> Int {\n    for item in items {\n        item = 1\n    }\n    0\n}\n";

    assert_eq!(
        refusal(source).message(),
        "`item` is not a `var`, so it is never assigned to"
    );
}

#[test]
fn a_constructor_is_refused_rather_than_compiled_into_a_store_with_nowhere_to_write() {
    let source = "fn go() -> Int {\n    Marker = Marker\n    1\n}\n\ntype Marker = {\n}\n";

    assert_eq!(
        refusal(source).message(),
        "`Marker` is not a `var`, so it is never assigned to"
    );
}

#[test]
fn a_plus_equals_names_a_var_the_same_way_an_equals_does() {
    let source = "fn count() -> Int {\n    total := 0\n    total += 2\n    total\n}\n";

    assert_eq!(
        refusal(source).message(),
        "`total` is not a `var`, so it is never assigned to"
    );
}

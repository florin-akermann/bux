//! `docs/specs/arguments.md`: which calls name their arguments, and which names they write.

use crate::common::{inferred, refusal};

/// A module whose `main` calls `rename` the way `call` writes it.
///
/// `rename` gives both of its parameters the type `String`, so nothing but the names holds its
/// two arguments apart, and every call of it here is one the rule reaches.
fn calling_rename(call: &str) -> String {
    format!(
        "fn main() -> String {{\n    {call}\n}}\n\n\
         fn rename(from: String, to: String) -> String {{\n    from + to\n}}\n"
    )
}

#[test]
fn a_call_of_a_declaration_that_repeats_a_type_names_its_arguments() {
    inferred(&calling_rename("rename(from: \"old\", to: \"new\")"));
}

#[test]
fn a_call_that_passes_them_in_order_names_the_type_the_declaration_repeats() {
    let error = refusal(&calling_rename("rename(\"old\", \"new\")"));

    assert_eq!(
        error.message(),
        "`rename` gives two parameters the type `String`, so this call names its arguments"
    );
}

#[test]
fn the_refusal_says_when_a_call_has_to_name_them() {
    let error = refusal(&calling_rename("rename(\"old\", \"new\")"));

    assert_eq!(
        error.help(),
        "a call names its arguments when the declaration gives two parameters one type"
    );
}

#[test]
fn a_name_written_for_another_parameter_says_which_one_belonged_there() {
    let error = refusal(&calling_rename("rename(to: \"new\", from: \"old\")"));

    assert_eq!(
        error.message(),
        "this argument is named `to`, and the parameter here is `from`"
    );
}

#[test]
fn the_refusal_says_that_naming_does_not_reorder_a_call() {
    let error = refusal(&calling_rename("rename(to: \"new\", from: \"old\")"));

    assert_eq!(
        error.help(),
        "arguments are named in the order the declaration lists its parameters"
    );
}

#[test]
fn a_name_no_parameter_has_is_refused_as_the_one_that_belonged_there() {
    let error = refusal(&calling_rename("rename(source: \"old\", to: \"new\")"));

    assert_eq!(
        error.message(),
        "this argument is named `source`, and the parameter here is `from`"
    );
}

#[test]
fn the_number_of_arguments_is_settled_before_which_of_them_is_which() {
    let error = refusal(&calling_rename("rename(\"old\")"));

    assert_eq!(
        error.message(),
        "`rename` takes 2 arguments but 1 was given"
    );
}

#[test]
fn a_declaration_whose_parameter_types_all_differ_may_be_called_either_way() {
    inferred(&repeating("    label(\"x\", 1)"));
    inferred(&repeating("    label(text: \"x\", times: 1)"));
}

/// A module whose `main` calls `label`, whose two parameters have two types.
fn repeating(call: &str) -> String {
    format!(
        "fn main() -> String {{\n{call}\n}}\n\n\
         fn label(text: String, times: Int) -> String {{\n    text\n}}\n"
    )
}

#[test]
fn an_unwritten_signature_counts_exactly_as_one_the_author_wrote_out() {
    let inferred_twice = "fn main() -> Int {\n    join(1, 2)\n}\n\n\
                          fn join(first, second) -> Int {\n    first + second\n}\n";

    let error = refusal(inferred_twice);

    assert_eq!(
        error.message(),
        "`join` gives two parameters the type `Int`, so this call names its arguments"
    );
}

#[test]
fn a_type_parameter_counts_as_a_type_the_declaration_repeats() {
    let generic = "fn main() -> Int {\n    pair(1, 2)\n}\n\n\
                   fn pair<T>(first: T, second: T) -> T {\n    first\n}\n";

    let error = refusal(generic);

    assert_eq!(
        error.message(),
        "`pair` gives two parameters the type `T`, so this call names its arguments"
    );
}

#[test]
fn two_type_parameters_are_two_types_however_a_call_instantiates_them() {
    let generic = "fn main() -> Int {\n    apply(1, 2)\n}\n\n\
                   fn apply<T, U>(value: T, other: U) -> T {\n    value\n}\n";

    inferred(generic);
}

#[test]
fn a_call_written_as_an_argument_is_held_to_the_rule_like_any_other() {
    let error = refusal(&calling_rename(
        "rename(from: rename(\"a\", \"b\"), to: \"new\")",
    ));

    assert_eq!(
        error.message(),
        "`rename` gives two parameters the type `String`, so this call names its arguments"
    );
}

/// Every place a statement or an expression can hold a call, each holding one that is refused.
const NESTED: [&str; 8] = [
    "    rename(\"old\", \"new\")",
    "    said := rename(\"old\", \"new\")\n    said",
    "    _ = rename(\"old\", \"new\")\n    \"\"",
    "    return rename(\"old\", \"new\")",
    "    rename(\"old\", \"new\") + \"\"",
    "    if true {\n        rename(\"old\", \"new\")\n    } else {\n        \"\"\n    }",
    "    for word in words {\n        _ = rename(word, \"new\")\n    }\n    \"\"",
    "    match count > 1 {\n        true => rename(\"old\", \"new\")\n        false => \"\"\n    }",
];

#[test]
fn a_call_is_held_to_the_rule_wherever_the_body_writes_it() {
    for body in NESTED {
        let source = format!(
            "fn main(count: Int, words: List<String>) -> String {{\n{body}\n}}\n\n\
             fn rename(from: String, to: String) -> String {{\n    from + to\n}}\n"
        );

        assert_eq!(
            refusal(&source).message(),
            "`rename` gives two parameters the type `String`, so this call names its arguments",
            "{body}"
        );
    }
}

#[test]
fn a_constructor_carries_its_values_in_order_and_has_no_names_to_write() {
    let built = "fn main() -> Range {\n    Range(0, 10)\n}\n\n\
                 type Range = Range(Int, Int)\n";

    inferred(built);
}

#[test]
fn a_constructor_call_that_writes_names_is_refused_rather_than_read_in_order() {
    let named = "fn main() -> Range {\n    Range(len: 10, start: 0)\n}\n\n\
                 type Range = Range(Int, Int)\n";

    let error = refusal(named);

    assert_eq!(
        error.message(),
        "`Range` is a constructor, so it carries its values in order and names none"
    );
    assert_eq!(
        error.help(),
        "a variant whose values want names declares them as fields and is built as a record"
    );
}

#[test]
fn a_prelude_function_is_not_one_this_module_declares() {
    inferred("fn main() -> Int {\n    or(Some(1), 0)\n}\n");
}

#[test]
fn a_prelude_call_that_writes_names_is_refused_because_nothing_here_checks_them() {
    let error = refusal("fn main() -> Int {\n    or(value: Some(1), fallback: 0)\n}\n");

    assert_eq!(
        error.message(),
        "`or` comes from the prelude, which declares no parameter names to write"
    );
    assert_eq!(
        error.help(),
        "only a call of a function this module declares names its arguments"
    );
}

#[test]
fn the_types_of_the_arguments_are_settled_before_the_call_is_asked_to_name_them() {
    let error = refusal(&calling_rename("rename(1, 2)"));

    assert_eq!(error.message(), "expected `String`, found `Int`");
}

#[test]
fn a_swapped_pair_of_one_type_is_still_named_where_it_is_written() {
    let error = refusal(&calling_rename("rename(to: \"new\", from: \"old\")"));

    assert_eq!(
        error.message(),
        "this argument is named `to`, and the parameter here is `from`"
    );
}

//! The behaviours of `docs/specs/api-surface.md`: what is on a page, in what order, written how.

use crate::common::page;

#[test]
fn a_function_is_on_the_page_as_its_signature_and_nothing_after_it() {
    let source = "fn shared(total: Int, people: Int) -> Int {\n    or(total / people, 0)\n}\n";

    assert_eq!(page(source), "fn shared(total: Int, people: Int) -> Int\n");
}

#[test]
fn a_function_that_takes_nothing_still_says_what_it_gives_back() {
    let source = "fn answer() -> Int {\n    7\n}\n";

    assert_eq!(page(source), "fn answer() -> Int\n");
}

#[test]
fn a_function_keeps_the_type_parameters_it_declares() {
    let source = concat!(
        "fn first<A, B>(pair: Pair<A, B>) -> A {\n    pair.left\n}\n\n",
        "type Pair<A, B> = {\n    left: A\n    right: B\n}\n"
    );

    assert!(
        page(source).starts_with("fn first<A, B>(pair: Pair<A, B>) -> A\n"),
        "{}",
        page(source)
    );
}

#[test]
fn a_type_the_author_left_to_inference_is_the_one_the_page_prints() {
    let source = "fn twice(n) {\n    n + n\n}\n";

    assert_eq!(page(source), "fn twice(n: Int) -> Int\n");
}

#[test]
fn a_type_nothing_settled_is_printed_as_an_underscore() {
    let source = "fn pick(a, b) {\n    a\n}\n";

    assert_eq!(page(source), "fn pick(a: _, b: _) -> _\n");
}

#[test]
fn a_record_declaration_is_on_the_page_with_every_field_it_declares() {
    let source = "type Basket = {\n    apples: Int\n    pears: Int\n}\n";

    assert_eq!(page(source), source);
}

#[test]
fn an_algebraic_data_type_is_on_the_page_with_every_variant_it_declares() {
    let source = "type Order =\n    | Unplaced\n    | Placed(Basket)\n\ntype Basket = {\n    apples: Int\n}\n";

    assert_eq!(page(source), source);
}

#[test]
fn the_page_lists_declarations_in_the_order_the_file_declares_them() {
    let source = concat!(
        "fn picked(order: Order) -> Int {\n",
        "    match order {\n        Unplaced => 0\n        Placed(count) => count\n    }\n}\n\n",
        "type Order =\n    | Unplaced\n    | Placed(Int)\n"
    );

    assert_eq!(
        page(source),
        "fn picked(order: Order) -> Int\n\ntype Order =\n    | Unplaced\n    | Placed(Int)\n"
    );
}

#[test]
fn an_import_brings_a_name_in_rather_than_putting_one_out_so_it_is_not_on_the_page() {
    let source = "import io\n\nfn answer() -> Int {\n    7\n}\n";

    assert_eq!(page(source), "fn answer() -> Int\n");
}

#[test]
fn a_comment_is_not_a_name_so_no_comment_reaches_the_page() {
    let source =
        "/// How much an order comes to.\nfn picked() -> Int {\n    // counted once\n    7\n}\n";

    assert_eq!(page(source), "fn picked() -> Int\n");
}

#[test]
fn a_comment_above_a_type_declaration_is_left_off_the_page_with_every_other_comment() {
    let source = "/// A basket of fruit.\ntype Basket = {\n    apples: Int\n}\n";

    assert_eq!(page(source), "type Basket = {\n    apples: Int\n}\n");
}

#[test]
fn a_parameter_inference_made_a_function_is_printed_the_way_a_diagnostic_prints_one() {
    let source = "fn apply(f, n) {\n    f(n)\n}\n";

    assert_eq!(page(source), "fn apply(f: (_) -> _, n: _) -> _\n");
}

#[test]
fn a_module_that_declares_nothing_has_an_empty_page() {
    assert_eq!(page("import io\n"), "");
}

#[test]
fn the_page_puts_exactly_one_blank_line_between_two_stanzas() {
    let source = concat!(
        "fn answer() -> Int {\n    7\n}\n\n",
        "fn named() -> String {\n    \"a\"\n}\n\n",
        "type Basket = {\n    apples: Int\n}\n"
    );

    assert_eq!(
        page(source),
        "fn answer() -> Int\n\nfn named() -> String\n\ntype Basket = {\n    apples: Int\n}\n"
    );
}

#[test]
fn a_module_that_holds_a_hole_has_a_page_because_a_hole_is_well_typed() {
    let source = "fn counted() -> Int {\n    todo(\"count them\")\n}\n";

    assert_eq!(page(source), "fn counted() -> Int\n");
}

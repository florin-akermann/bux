//! The instances of a type travel with the type, into every module that reaches it.
//!
//! `docs/specs/modules.md` states the rule, which `docs/design.md` section 8 settles: an instance
//! is declared beside its trait or beside its type, so a module that reaches `money.Money` reaches
//! every instance `money` gives `Money`, however it came by a value of it.

use lumen_types::Imported;

use crate::common::{Offered, inferred_reaching, inferred_type_reaching};
use crate::common::{inferred, offering, reaching_the_library, refusal_reaching};

/// A module declaring a type with a derived `Eq`, `Ord`, `Hash`, and `Show`, and a written `Add`.
const MONEY: &str = concat!(
    "fn cents(count: Int) -> Money {\n    Money(count)\n}\n\n",
    "derive Eq, Ord, Hash, Show for Money\n\n",
    "instance Add<Money> {\n",
    "    fn add(left: Money, right: Money) -> Money {\n",
    "        match left {\n",
    "            Money(one) => match right {\n",
    "                Money(other) => Money(one + other)\n",
    "            }\n",
    "        }\n",
    "    }\n",
    "}\n\n",
    "instance IntegerLiteral<Money> {\n",
    "    fn lowest() -> Int {\n        0\n    }\n\n",
    "    fn highest() -> Int {\n        100\n    }\n\n",
    "    fn from_literal(literal: Int) -> Money {\n        Money(literal)\n    }\n",
    "}\n\n",
    "type Money = Money(Int)\n"
);

/// A module declaring a type and no instance of it at all.
const PLAIN: &str = "fn plain() -> Plain {\n    Plain(1)\n}\n\ntype Plain = Plain(Int)\n";

/// What `source` puts out under the name `money`, which is the name each test imports.
fn offered(source: &str) -> Imported {
    offering(&Offered {
        module: "money",
        source,
    })
}

/// A module importing `money` and writing `written` as the body of `main`.
fn reaching(written: &str) -> String {
    format!("import money\n\nfn main() -> () {{\n    {written}\n}}\n")
}

/// The type of `expression`, where a module reaching `money` writes it as the body of `main`.
fn type_of_money(expression: &str) -> String {
    let source = reaching(&format!("_ = {expression}"));
    inferred_type_reaching(&source, expression, 1, &offered(MONEY))
}

#[test]
fn an_equality_over_an_imported_type_is_written_by_the_instance_its_module_derived() {
    assert_eq!(type_of_money("money.cents(1) == money.cents(2)"), "Bool");
}

#[test]
fn an_order_and_an_addition_over_an_imported_type_reach_the_instances_its_module_declares() {
    assert_eq!(
        type_of_money("money.cents(1) + money.cents(2) < money.cents(4)"),
        "Bool"
    );
    assert_eq!(
        type_of_money("money.cents(1) + money.cents(2)"),
        "money.Money"
    );
}

#[test]
fn a_constraint_settled_at_an_imported_type_is_answered_by_the_instance_of_its_module() {
    let source = concat!(
        "import money\n\n",
        "fn main() -> () {\n    _ = is_same(one: money.cents(1), other: money.cents(1))\n}\n\n",
        "fn is_same<T: Eq<T>>(one: T, other: T) -> Bool {\n    one == other\n}\n"
    );

    inferred_reaching(source, &offered(MONEY));
}

#[test]
fn a_whole_number_takes_an_imported_type_whose_module_gives_it_an_integer_literal() {
    let source = "import money\n\nfn paid() -> money.Money {\n    42\n}\n";

    assert_eq!(
        inferred_type_reaching(source, "42", 1, &offered(MONEY)),
        "money.Money"
    );
}

#[test]
fn a_derive_here_reaches_the_instances_of_an_imported_field() {
    let source = concat!(
        "import money\n\n",
        "derive Eq, Hash for Wallet\n\n",
        "type Wallet = Wallet(money.Money)\n"
    );

    inferred_reaching(source, &offered(MONEY));
}

#[test]
fn a_map_is_keyed_by_an_imported_type_whose_module_gives_it_eq_and_hash() {
    let source = concat!(
        "import map\nimport money\n\n",
        "fn main() -> () {\n    _ = map.insert(map.empty(), money.cents(1), 1)\n}\n"
    );
    let surface = inferred(MONEY).surface().clone();
    let imported = reaching_the_library().offering("money", surface);

    inferred_reaching(source, &imported);
}

#[test]
fn an_imported_type_its_module_gives_no_instance_is_refused_where_the_operator_is_written() {
    let source = reaching("_ = money.plain() == money.plain()");

    assert_eq!(
        refusal_reaching(&source, &offered(PLAIN)).message(),
        "`money.Plain` has no `Eq`, so `==` is not written over it"
    );
}

#[test]
fn a_type_reached_through_a_module_that_is_not_imported_still_has_its_instances() {
    let relaying = "import money\n\nfn got() -> money.Money {\n    money.cents(1)\n}\n";
    let imported = offered(MONEY);
    let surface = inferred_reaching(relaying, &imported).surface().clone();
    let relayed = imported.offering("relay", surface);
    let source = "import relay\n\nfn main() -> () {\n    _ = relay.got() == relay.got()\n}\n";

    inferred_reaching(source, &relayed);
}

/// A module whose instance of `Eq` is constrained by a trait it keeps to itself.
const KEEPING: &str = concat!(
    "fn boxed(value: Int) -> Box<Int> {\n    Box(value)\n}\n\n",
    "instance<T: Secret<T>> Eq<Box<T>> {\n",
    "    fn is_equal(one: Box<T>, other: Box<T>) -> Bool {\n        true\n    }\n}\n\n",
    "instance Secret<Int> {\n    fn told(value: Int) -> Int {\n        value\n    }\n}\n\n",
    "trait Secret<T> {\n    fn told(value: T) -> Int\n}\n\n",
    "type Box<T> = Box(T)\n"
);

#[test]
fn an_instance_constrained_by_a_trait_its_module_keeps_stays_in_that_module() {
    let source = concat!(
        "import money\n\n",
        "fn main() -> () {\n    _ = money.boxed(1) == money.boxed(1)\n}\n\n",
        "instance Secret<Int> {\n    fn told(value: Int) -> Int {\n        value\n    }\n}\n\n",
        "trait Secret<T> {\n    fn told(value: T) -> Int\n}\n"
    );

    assert_eq!(
        refusal_reaching(source, &offered(KEEPING)).message(),
        "`money.Box<Int>` has no `Eq`, so `==` is not written over it"
    );
}

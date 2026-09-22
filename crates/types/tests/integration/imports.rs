//! What a module reaches through an import, and what an imported module keeps to itself.
//!
//! `docs/specs/modules.md` states the rule: a loaded module offers every function and every type
//! it declares, each reached through the name the import brings into scope.

use lumen_ast::Span;
use lumen_types::{GenericUse, Imported};

use crate::common::{Offered, inferred_reaching, inferred_type_reaching};
use crate::common::{offering, refusal_reaching};

/// A module declaring one function, which is what another module imports it for.
const GREETING: &str = "fn hello(name: String) -> String {\n    \"hi \" + name\n}\n";

/// A module declaring a type as well, which is what another module reaches through its name.
const WRAPPING: &str = "fn wrapped(name: String) -> Greeting {\n    Greeting(name)\n}\n\ntype Greeting = Greeting(String)\n";

/// A module declaring a record, which another module builds and reads the fields of.
const USER: &str =
    "fn named(user: User) -> String {\n    user.name\n}\n\ntype User = {\n    name: String\n}\n";

/// A module declaring an algebraic data type, which another module matches values of.
const PAYMENT: &str = "fn sent(how: String) -> Payment {\n    Sent(how)\n}\n\ntype Payment =\n    | Sent(String)\n    | Pending\n";

/// What `source` puts out under the name `greeting`, which is the name each test imports.
fn offered(source: &str) -> Imported {
    offering(&Offered {
        module: "greeting",
        source,
    })
}

/// A module importing `greeting` and writing `written` as the body of `main`.
fn reaching(written: &str) -> String {
    format!("import greeting\n\nfn main() -> () {{\n    {written}\n}}\n")
}

#[test]
fn a_function_of_an_imported_module_has_the_type_that_module_gave_it() {
    let source = reaching("_ = greeting.hello(\"world\")");
    let imported = offered(GREETING);

    assert_eq!(
        inferred_type_reaching(&source, "greeting.hello", 1, &imported),
        "(String) -> String"
    );
}

#[test]
fn what_a_function_of_an_imported_module_gives_back_is_what_its_result_says() {
    let source = "import greeting\n\nfn greet() -> String {\n    greeting.hello(\"world\")\n}\n";

    inferred_reaching(source, &offered(GREETING));
}

#[test]
fn an_argument_passed_through_an_import_is_held_to_the_parameter_that_module_declared() {
    let source = reaching("_ = greeting.hello(7)");

    assert_eq!(
        refusal_reaching(&source, &offered(GREETING)).message(),
        "expected `String`, found `Int`"
    );
}

#[test]
fn a_name_an_imported_module_does_not_declare_is_refused_where_it_is_written() {
    let source = reaching("_ = greeting.farewell(\"world\")");

    assert_eq!(
        refusal_reaching(&source, &offered(GREETING)).message(),
        "`greeting` declares no `farewell`"
    );
}

#[test]
fn a_type_the_imported_module_declares_is_named_by_the_module_it_is_reached_through() {
    let source = reaching("_ = greeting.wrapped(\"world\")");

    assert_eq!(
        inferred_type_reaching(&source, "greeting.wrapped", 1, &offered(WRAPPING)),
        "(String) -> greeting.Greeting"
    );
}

#[test]
fn a_type_of_another_module_is_named_that_way_however_deep_in_the_signature_it_sits() {
    let greeting =
        "fn taken(held: List<Greeting>) -> Int {\n    1\n}\n\ntype Greeting = Greeting(String)\n";
    let source = reaching("_ = greeting.taken([])");

    assert_eq!(
        inferred_type_reaching(&source, "greeting.taken", 1, &offered(greeting)),
        "(List<greeting.Greeting>) -> Int"
    );
}

#[test]
fn a_constructor_of_another_module_builds_that_module_s_type() {
    let source = reaching("_ = greeting.Greeting(\"world\")");

    assert_eq!(
        inferred_type_reaching(
            &source,
            "greeting.Greeting(\"world\")",
            1,
            &offered(WRAPPING)
        ),
        "greeting.Greeting"
    );
}

#[test]
fn a_type_of_another_module_is_written_where_a_signature_writes_a_type() {
    let source = concat!(
        "import greeting\n\n",
        "fn held(value: greeting.Greeting) -> greeting.Greeting {\n    value\n}\n"
    );

    inferred_reaching(source, &offered(WRAPPING));
}

#[test]
fn a_record_of_another_module_is_built_by_its_name_and_read_field_by_field() {
    let source = concat!(
        "import greeting\n\n",
        "fn named() -> String {\n",
        "    user := greeting.User { name: \"world\" }\n",
        "    user.name\n",
        "}\n"
    );

    inferred_reaching(source, &offered(USER));
}

#[test]
fn a_variant_of_another_module_is_matched_by_the_name_it_is_reached_through() {
    let source = concat!(
        "import greeting\n\n",
        "fn said(payment: greeting.Payment) -> String {\n",
        "    match payment {\n",
        "        greeting.Sent(how) => how\n",
        "        greeting.Pending => \"pending\"\n",
        "    }\n",
        "}\n"
    );

    inferred_reaching(source, &offered(PAYMENT));
}

#[test]
fn a_generic_type_of_another_module_takes_its_arguments_after_the_whole_name() {
    let greeting =
        "fn held<T>(value: T) -> Held<T> {\n    Held(value)\n}\n\ntype Held<T> = Held(T)\n";
    let source = concat!(
        "import greeting\n\n",
        "fn taken(value: greeting.Held<Int>) -> greeting.Held<Int> {\n    value\n}\n"
    );

    inferred_reaching(source, &offered(greeting));
}

#[test]
fn a_use_of_a_generic_type_of_another_module_is_named_as_written_when_it_counts_wrong() {
    let greeting =
        "fn held<T>(value: T) -> Held<T> {\n    Held(value)\n}\n\ntype Held<T> = Held(T)\n";
    let source = concat!(
        "import greeting\n\n",
        "fn taken(value: greeting.Held<Int, Int>) -> Int {\n    1\n}\n"
    );

    assert_eq!(
        refusal_reaching(source, &offered(greeting)).message(),
        "`greeting.Held` takes 1 type argument but 2 were given"
    );
}

#[test]
fn a_type_of_a_module_this_one_does_not_import_is_read_as_the_type_it_was_declared_as() {
    let source = "import relay\n\nfn named() -> String {\n    relay.got().name\n}\n";

    assert_eq!(
        inferred_type_reaching(source, "relay.got()", 1, &relayed()),
        "greeting.User"
    );
    assert_eq!(
        inferred_type_reaching(source, "relay.got().name", 1, &relayed()),
        "String"
    );
}

/// `relay` offers a function giving back a `greeting.User`, and `greeting` is not imported.
///
/// A module reaches what it imports and holds what those give it, so a value of `greeting.User`
/// arrives here through `relay` however little this module has to do with `greeting`.
fn relayed() -> Imported {
    let relaying = concat!(
        "import greeting\n\n",
        "fn got() -> greeting.User {\n    greeting.User { name: \"world\" }\n}\n"
    );
    let imported = offered(USER);
    let surface = inferred_reaching(relaying, &imported).surface().clone();
    imported.offering("relay", surface)
}

#[test]
fn a_type_the_imported_module_does_not_declare_is_refused_where_it_is_written() {
    let source = concat!(
        "import greeting\n\n",
        "fn held(value: greeting.Farewell) -> Int {\n    1\n}\n"
    );

    assert_eq!(
        refusal_reaching(source, &offered(WRAPPING)).message(),
        "`greeting` declares no `Farewell`"
    );
}

#[test]
fn a_type_of_another_module_has_no_instance_here_however_that_module_came_by_one() {
    let greeting = concat!(
        "derive Eq for Greeting\n\n",
        "fn wrapped(name: String) -> Greeting {\n    Greeting(name)\n}\n\n",
        "type Greeting = Greeting(String)\n"
    );
    let source = reaching("_ = greeting.Greeting(\"a\") == greeting.Greeting(\"b\")");

    assert_eq!(
        refusal_reaching(&source, &offered(greeting)).message(),
        "`greeting.Greeting` has no `Eq`, so `==` is not written over it"
    );
}

#[test]
fn a_module_may_declare_a_type_and_still_offer_what_is_written_without_it() {
    let greeting = "fn hello(name: String) -> String {\n    \"hi \" + name\n}\n\ntype Greeting = Greeting(String)\n";
    let source = reaching("_ = greeting.hello(\"world\")");

    inferred_reaching(&source, &offered(greeting));
}

#[test]
fn a_generic_function_is_reached_through_an_import_at_the_types_the_use_settles() {
    let greeting = "fn held<T>(value: T) -> T {\n    value\n}\n";
    let source = reaching("_ = greeting.held(1)");

    assert_eq!(
        inferred_type_reaching(&source, "greeting.held", 1, &offered(greeting)),
        "(Int) -> Int"
    );
}

#[test]
fn a_use_of_an_imported_generic_says_what_it_settled_each_type_parameter_on() {
    let greeting = "fn held<T>(value: T) -> T {\n    value\n}\n";
    let source = reaching("_ = greeting.held(1)");

    assert_eq!(settled_by(&source, greeting), ["Int"]);
}

#[test]
fn a_use_of_an_imported_generic_is_written_as_the_declaration_is_at_that_set() {
    let greeting = "fn held<T>(value: T) -> T {\n    value\n}\n";
    let source = reaching("_ = greeting.held(1)");

    assert_eq!(written_as(&source, greeting), "(Int) -> Int");
}

#[test]
fn a_function_whose_type_nothing_settled_is_generic_and_is_reached_all_the_same() {
    let greeting = "fn held(value) {\n    value\n}\n";
    let source = reaching("_ = greeting.held(1)");

    assert_eq!(
        inferred_type_reaching(&source, "greeting.held", 1, &offered(greeting)),
        "(Int) -> Int"
    );
    assert_eq!(settled_by(&source, greeting), [] as [&str; 0]);
}

#[test]
fn a_stand_in_inference_made_is_erased_rather_than_written_at_the_type_it_met() {
    let greeting = "fn held(value) {\n    value\n}\n";
    let source = reaching("_ = greeting.held(1)");

    assert_eq!(written_as(&source, greeting), "(_) -> _");
}

/// A module whose generic writes a constraint, answered by an instance it declares itself.
const SAME: &str = concat!(
    "derive Eq for Greeting\n\n",
    "fn is_same<T: Eq<T>>(one: T, other: T) -> Bool {\n    one == other\n}\n\n",
    "fn wrapped(name: String) -> Greeting {\n    Greeting(name)\n}\n\n",
    "type Greeting = Greeting(String)\n"
);

#[test]
fn a_constraint_on_an_imported_generic_is_answered_by_the_prelude_s_instances() {
    let source = reaching("_ = greeting.is_same(\"a\", \"b\")");

    inferred_reaching(&source, &offered(SAME));
}

#[test]
fn a_constraint_settled_at_a_type_of_the_declaring_module_has_no_instance_here() {
    let source = reaching("_ = greeting.is_same(greeting.wrapped(\"a\"), greeting.wrapped(\"b\"))");

    assert_eq!(
        refusal_reaching(&source, &offered(SAME)).message(),
        "`greeting.Greeting` has no instance of `Eq`"
    );
}

#[test]
fn an_instance_this_module_declares_answers_a_constraint_on_an_imported_generic() {
    let source = concat!(
        "import greeting\n\n",
        "fn main() -> () {\n",
        "    _ = greeting.is_same(Kept(\"a\"), Kept(\"b\"))\n",
        "}\n\n",
        "derive Eq for Kept\n\n",
        "type Kept = Kept(String)\n"
    );

    inferred_reaching(source, &offered(SAME));
}

/// A module whose generic writes a constraint over a trait it declares itself.
const LABELLING: &str = concat!(
    "fn labelled<T: Named<T>>(value: T) -> String {\n    named_as(value)\n}\n\n",
    "instance Named<String> {\n",
    "    fn named_as(value: String) -> String {\n        value\n    }\n}\n\n",
    "trait Named<T> {\n    fn named_as(value: T) -> String\n}\n"
);

#[test]
fn a_constraint_over_a_trait_the_other_module_keeps_to_itself_is_answered_by_nothing_here() {
    let source = concat!(
        "import greeting\n\n",
        "fn main() -> () {\n",
        "    _ = greeting.labelled(Kept(\"a\"))\n",
        "}\n\n",
        "type Kept = Kept(String)\n"
    );

    assert_eq!(
        refusal_reaching(source, &offered(LABELLING)).message(),
        "`greeting.labelled` requires `Named`, which `greeting` declares and nothing here names"
    );
}

/// What the one use of `greeting.held` in `source` settled each type parameter on.
fn settled_by(source: &str, greeting: &str) -> Vec<String> {
    reached_by(source, greeting)
        .settled()
        .iter()
        .map(ToString::to_string)
        .collect()
}

/// The type the method that one use of `greeting.held` reaches is written with.
fn written_as(source: &str, greeting: &str) -> String {
    reached_by(source, greeting).written_as().to_string()
}

/// What the one use of `greeting.held` in `source` reaches.
fn reached_by(source: &str, greeting: &str) -> GenericUse {
    let at = source
        .find("held")
        .expect("the source under test writes one use of `greeting.held`");
    inferred_reaching(source, &offered(greeting))
        .generic_reached(Span::new(at, "held".len()))
        .expect("a use of a generic another module declares says what it reaches")
        .clone()
}

#[test]
fn two_calls_of_one_imported_function_each_read_the_type_freshly() {
    let source = "import greeting\n\nfn main() -> () {\n    _ = greeting.hello(\"one\")\n    _ = greeting.hello(\"two\")\n}\n";

    inferred_reaching(source, &offered(GREETING));
}

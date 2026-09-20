//! What a module reaches through an import, and what an imported module keeps to itself.
//!
//! `docs/specs/modules.md` states the rule: a loaded module offers every function it declares,
//! and a type it declares stays its own, so a signature naming one is refused where it is
//! reached.

use lumen_types::Imported;

use crate::common::{Offered, inferred_reaching, inferred_type_reaching};
use crate::common::{offering, refusal_reaching};

/// A module declaring one function, which is what another module imports it for.
const GREETING: &str = "fn hello(name: String) -> String {\n    \"hi \" + name\n}\n";

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
fn a_type_declared_by_the_imported_module_is_kept_to_itself() {
    let greeting = "fn wrapped(name: String) -> Greeting {\n    Greeting(name)\n}\n\ntype Greeting = Greeting(String)\n";
    let source = reaching("_ = greeting.wrapped(\"world\")");
    let error = refusal_reaching(&source, &offered(greeting));

    assert_eq!(
        error.message(),
        "`greeting.wrapped` names `Greeting`, which `greeting` keeps to itself"
    );
    assert_eq!(
        error.help(),
        "a signature written in types both modules have is what one module offers another"
    );
}

#[test]
fn a_type_kept_to_itself_is_found_however_deep_in_the_signature_it_is_written() {
    let greeting =
        "fn taken(held: List<Greeting>) -> Int {\n    1\n}\n\ntype Greeting = Greeting(String)\n";
    let source = reaching("_ = greeting.taken([])");

    assert_eq!(
        refusal_reaching(&source, &offered(greeting)).message(),
        "`greeting.taken` names `Greeting`, which `greeting` keeps to itself"
    );
}

#[test]
fn a_module_may_declare_a_type_and_still_offer_what_is_written_without_it() {
    let greeting = "fn hello(name: String) -> String {\n    \"hi \" + name\n}\n\ntype Greeting = Greeting(String)\n";
    let source = reaching("_ = greeting.hello(\"world\")");

    inferred_reaching(&source, &offered(greeting));
}

#[test]
fn a_generic_function_is_written_where_it_is_declared_and_not_reached_through_an_import() {
    let greeting = "fn held<T>(value: T) -> T {\n    value\n}\n";
    let source = reaching("_ = greeting.held(1)");
    let error = refusal_reaching(&source, &offered(greeting));

    assert_eq!(
        error.message(),
        "`greeting.held` is generic, so `greeting` alone writes it"
    );
    assert_eq!(
        error.help(),
        "write it in the module that reaches it, or give it a signature at one set of types"
    );
}

#[test]
fn a_function_whose_type_nothing_settled_is_generic_and_is_not_reached_through_an_import() {
    let greeting = "fn held(value) {\n    value\n}\n";
    let source = reaching("_ = greeting.held(1)");
    let error = refusal_reaching(&source, &offered(greeting));

    assert_eq!(
        error.message(),
        "`greeting.held` is generic, so `greeting` alone writes it"
    );
}

#[test]
fn two_calls_of_one_imported_function_each_read_the_type_freshly() {
    let source = "import greeting\n\nfn main() -> () {\n    _ = greeting.hello(\"one\")\n    _ = greeting.hello(\"two\")\n}\n";

    inferred_reaching(source, &offered(GREETING));
}

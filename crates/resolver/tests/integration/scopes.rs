//! The scopes of `docs/specs/modules.md`: what each name in a module means.

use lumen_resolver::Namespace::{Type, Value};
use lumen_resolver::{Definition, DefinitionKind, Namespace, Origin};

use crate::common::{meaning, resolved};

#[test]
fn a_use_points_at_the_declaration_it_means() {
    let source = "fn two() -> Int {\n    one()\n}\n\nfn one() -> Int {\n    1\n}\n";

    let declared = meaning(source, Value, "one", 1).expect("the declaration names itself");
    assert_eq!(declared.kind, DefinitionKind::Function);
    assert_eq!(meaning(source, Value, "one", 2), Some(declared));
}

#[test]
fn a_function_may_call_one_declared_below_it() {
    let source = "fn first() -> Int {\n    second()\n}\n\nfn second() -> Int {\n    1\n}\n";

    assert_eq!(
        kind(source, Value, "second", 1),
        Some(DefinitionKind::Function)
    );
}

#[test]
fn a_newtype_names_a_type_and_a_constructor_without_the_two_clashing() {
    let source = "fn wrap(raw: Int) -> UserId {\n    UserId(raw)\n}\n\ntype UserId = UserId(Int)\n";

    assert_eq!(kind(source, Type, "UserId", 3), Some(DefinitionKind::Type));
    assert_eq!(
        kind(source, Value, "UserId", 4),
        Some(DefinitionKind::Constructor)
    );
}

#[test]
fn a_record_type_declares_a_type_and_the_value_it_is_built_with_at_one_place() {
    let source = "fn make() -> User {\n    User { id: 1 }\n}\n\ntype User = {\n    id: Int\n}\n";

    assert_eq!(kind(source, Type, "User", 3), Some(DefinitionKind::Type));
    assert_eq!(
        kind(source, Value, "User", 3),
        Some(DefinitionKind::Constructor)
    );
}

#[test]
fn a_parameter_is_in_scope_in_the_body() {
    let source = "fn double(value: Int) -> Int {\n    value + value\n}\n";

    assert_eq!(
        kind(source, Value, "value", 2),
        Some(DefinitionKind::Parameter)
    );
}

#[test]
fn a_binding_is_in_scope_after_the_statement_that_makes_it() {
    let source = "fn f() -> Int {\n    total := 1\n    total + 1\n}\n";

    assert_eq!(kind(source, Value, "total", 2), Some(DefinitionKind::Local));
}

#[test]
fn a_var_binds_a_name_that_may_be_assigned_to() {
    let source = "fn f() -> Int {\n    var total = 1\n    total + 1\n}\n";

    assert_eq!(
        kind(source, Value, "total", 2),
        Some(DefinitionKind::Variable)
    );
}

#[test]
fn a_type_parameter_is_in_scope_in_the_signature_that_declares_it() {
    let source = "fn identity<T>(value: T) -> T {\n    value\n}\n";

    assert_eq!(
        kind(source, Type, "T", 2),
        Some(DefinitionKind::TypeParameter)
    );
}

#[test]
fn the_prelude_is_in_scope_without_an_import() {
    let source = "fn ok() -> Result<Int, String> {\n    Ok(1)\n}\n";

    assert_eq!(
        meaning(source, Type, "Result", 1),
        Some(Definition {
            kind: DefinitionKind::Type,
            origin: Origin::Prelude
        })
    );
    assert_eq!(
        meaning(source, Value, "Ok", 1),
        Some(Definition {
            kind: DefinitionKind::Constructor,
            origin: Origin::Prelude
        })
    );
}

#[test]
fn an_imported_module_is_in_scope_under_its_own_name() {
    let source = "import io\n\nfn greet() {\n    io.print(\"hi\")\n}\n";

    assert_eq!(kind(source, Value, "io", 2), Some(DefinitionKind::Module));
}

#[test]
fn a_name_written_after_a_dot_is_not_looked_up_here() {
    let source = "import io\n\nfn greet() {\n    io.print(\"hi\")\n}\n";

    assert_eq!(meaning(source, Value, "print", 1), None);
}

#[test]
fn a_field_of_a_record_literal_is_not_looked_up_here() {
    let source = "fn make() -> User {\n    User { id: 1 }\n}\n\ntype User = {\n    id: Int\n}\n";

    assert_eq!(meaning(source, Value, "id", 2), None);
}

#[test]
fn a_bare_pattern_that_names_a_variant_is_a_use_and_not_a_binding() {
    let source = "fn describe(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Failed(reason) => reason\n    }\n}\n\ntype Payment =\n    | Pending\n    | Failed(String)\n";

    assert_eq!(
        kind(source, Value, "Pending", 2),
        Some(DefinitionKind::Constructor)
    );
    assert_eq!(
        kind(source, Value, "reason", 1),
        Some(DefinitionKind::Local)
    );
}

#[test]
fn a_record_pattern_binds_each_field_it_names() {
    let source = "fn describe(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Authorized { id } => id\n    }\n}\n\ntype Payment =\n    | Pending\n    | Authorized {\n        id: String\n    }\n";

    assert_eq!(kind(source, Value, "id", 2), Some(DefinitionKind::Local));
}

#[test]
fn the_tree_comes_through_resolution_unchanged() {
    let source = "fn f() -> Int {\n    1\n}\n";

    assert_eq!(resolved(source).program().items.len(), 1);
}

/// What kind of thing one occurrence of a name was declared as.
fn kind(
    source: &str,
    namespace: Namespace,
    written: &str,
    occurrence: usize,
) -> Option<DefinitionKind> {
    meaning(source, namespace, written, occurrence).map(|definition| definition.kind)
}

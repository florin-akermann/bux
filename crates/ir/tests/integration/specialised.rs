//! A generic another module declares, which that module writes and this one calls.
//!
//! `docs/specs/codegen.md` writes a generic once per set of types it is used at, in the class of
//! the module that declares it. A use in another module settles the set where it is written, so
//! the set is asked of that module and the method comes back under a name both work out alike.

use lumen_ir::{ClassName, Descriptor, Instruction, Lowered, MethodDescriptor};

use crate::common;

/// A module importing `holder` and writing `body` as the body of the one function it declares.
fn reaching(signature: &str, result: &str, body: &str) -> String {
    format!("import holder\n\nfn go({signature}) -> {result} {{\n    {body}\n}}\n")
}

/// The one call the body of `go` makes into `holder`, which is what the use reaches.
fn calling(source: &str) -> lumen_ir::MethodRef {
    reached_from(source, "go")
}

/// The one call the method `named` makes into `holder`, which is what the use reaches.
fn reached_from(source: &str, named: &str) -> lumen_ir::MethodRef {
    let lowered = common::lowered_reaching(source, &common::holder());
    common::body_of(&lowered, named)
        .instructions
        .iter()
        .find_map(|instruction| match instruction {
            Instruction::InvokeStatic(reached) if reached.class == ClassName::new("holder") => {
                Some(reached.clone())
            }
            _ => None,
        })
        .expect("a call into another module is a call of that module's class")
}

#[test]
fn a_use_of_another_module_s_generic_calls_that_module_s_class() {
    let reached = calling(&reaching("", "Int", "holder.held(1)"));

    assert_eq!(reached.class, ClassName::new("holder"));
}

#[test]
fn the_method_it_calls_is_named_for_the_set_of_types_the_use_settled() {
    let reached = calling(&reaching("", "Int", "holder.held(1)"));

    assert_eq!(reached.name, "held$Int");
}

#[test]
fn the_method_it_calls_takes_and_gives_back_what_that_set_settles_the_declaration_at() {
    let reached = calling(&reaching("", "Int", "holder.held(1)"));

    assert_eq!(
        reached.descriptor,
        MethodDescriptor::new(vec![Descriptor::Long], Some(Descriptor::Long))
    );
}

#[test]
fn the_module_writing_the_use_writes_no_method_for_it() {
    let lowered =
        common::lowered_reaching(&reaching("", "Int", "holder.held(1)"), &common::holder());

    assert!(!common::has_method(&lowered, "held$Int"));
    assert!(!common::has_method(&lowered, "held"));
}

#[test]
fn the_module_declaring_it_writes_the_method_the_use_asked_for() {
    let holder = common::holder_asked_by(&reaching("", "Int", "holder.held(1)"));

    assert_eq!(
        method_named(&holder, "held$Int"),
        Some(MethodDescriptor::new(
            vec![Descriptor::Long],
            Some(Descriptor::Long)
        ))
    );
}

#[test]
fn a_set_nothing_asked_for_is_written_by_nobody() {
    let holder = common::holder_asked_by(&reaching("", "Int", "holder.held(1)"));

    assert_eq!(method_named(&holder, "held$String"), None);
}

#[test]
fn two_uses_at_two_types_ask_for_two_methods() {
    let source = reaching("", "String", "_ = holder.held(1)\n    holder.held(\"two\")");

    let holder = common::holder_asked_by(&source);

    assert!(method_named(&holder, "held$Int").is_some());
    assert!(method_named(&holder, "held$String").is_some());
}

#[test]
fn two_uses_at_one_type_ask_for_the_one_method_once() {
    let source = reaching("", "Int", "_ = holder.held(1)\n    holder.held(2)");

    let holder = common::holder_asked_by(&source);

    assert_eq!(
        holder_methods(&holder)
            .filter(|name| *name == "held$Int")
            .count(),
        1
    );
}

#[test]
fn a_set_naming_a_type_of_the_asking_module_asks_under_that_module_s_name() {
    let source = concat!(
        "import holder\n\n",
        "fn go() -> Kept {\n    holder.held(Kept(\"a\"))\n}\n\n",
        "type Kept = Kept(String)\n"
    );

    let reached = calling(source);

    assert_eq!(reached.name, "held$demo$Kept");
    assert_eq!(
        reached.descriptor,
        MethodDescriptor::new(
            vec![Descriptor::reference("demo/Kept")],
            Some(Descriptor::reference("demo/Kept"))
        )
    );
}

#[test]
fn a_type_parameter_that_reaches_no_further_than_a_list_still_names_the_set() {
    let reached = calling(&reaching("", "Int", "holder.counted([1, 2])"));

    assert_eq!(reached.name, "counted$Int");
    assert_eq!(
        reached.descriptor,
        MethodDescriptor::new(vec![common::a_list()], Some(Descriptor::Long))
    );
}

#[test]
fn a_use_inside_a_generic_asks_for_what_the_set_that_generic_was_written_for_gives_it() {
    let source = concat!(
        "import holder\n\n",
        "fn go(value: Int) -> Int {\n    passed(value)\n}\n\n",
        "fn passed<T>(value: T) -> T {\n    holder.held(value)\n}\n"
    );

    let reached = reached_from(source, "passed$Int");

    assert_eq!(reached.name, "held$Int");
    assert_eq!(
        reached.descriptor,
        MethodDescriptor::new(vec![Descriptor::Long], Some(Descriptor::Long))
    );
}

#[test]
fn a_stand_in_inference_made_is_erased_rather_than_named_in_the_method() {
    let reached = calling(&reaching("", "Int", "holder.passed(1)"));

    assert_eq!(reached.name, "passed");
    assert_eq!(
        reached.descriptor,
        MethodDescriptor::new(vec![object()], Some(object()))
    );
}

#[test]
fn the_module_declaring_an_inferred_generic_writes_it_erased_too() {
    let holder = common::holder_asked_by(&reaching("", "Int", "holder.passed(1)"));

    assert_eq!(
        method_named(&holder, "passed"),
        Some(MethodDescriptor::new(vec![object()], Some(object())))
    );
}

/// `java.lang.Object`, which is what a stand-in nothing settled is carried by.
fn object() -> Descriptor {
    Descriptor::reference("java/lang/Object")
}

/// The descriptor of the one method of the `holder` class called `name`, where it writes one.
fn method_named(holder: &Lowered, name: &str) -> Option<MethodDescriptor> {
    common::class_of(holder, &ClassName::new("holder"))
        .methods
        .iter()
        .find(|method| method.name == name)
        .map(|method| method.descriptor.clone())
}

/// What every method of the `holder` class is called.
fn holder_methods(holder: &Lowered) -> impl Iterator<Item = &str> {
    common::class_of(holder, &ClassName::new("holder"))
        .methods
        .iter()
        .map(|method| method.name.as_str())
}

/// The uses a property is checked over, each with the type the call gives back.
const USES: [(&str, &str); 8] = [
    ("holder.held(1)", "Int"),
    ("holder.held(\"one\")", "String"),
    ("holder.held(Kept(\"a\"))", "Kept"),
    ("holder.held([2])", "List<Int>"),
    ("holder.passed(1)", "Int"),
    ("holder.counted([1])", "Int"),
    ("holder.counted([\"one\"])", "Int"),
    ("holder.counted([Kept(\"a\")])", "Int"),
];

/// `docs/specs/codegen.md`: the module declaring a generic writes every set a use asks it for.
///
/// The two sides work the method out apart: one from the types its use settled, the other from
/// the set it was asked for. The property is that they always arrive at the same method.
#[hegel::test]
fn every_method_a_use_asks_for_is_one_the_module_asked_writes(tc: hegel::TestCase) {
    let (use_of_it, result) = tc.draw(hegel::generators::sampled_from(&USES));
    let source = format!(
        "import holder\n\nfn go() -> {result} {{\n    {use_of_it}\n}}\n\ntype Kept = Kept(String)\n"
    );

    let reached = calling(&source);
    let holder = common::holder_asked_by(&source);

    assert_eq!(
        method_named(&holder, &reached.name),
        Some(reached.descriptor.clone()),
        "`holder` writes `{}` as `{use_of_it}` reaches it",
        reached.name
    );
}

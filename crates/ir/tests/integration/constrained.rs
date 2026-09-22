//! A generic constrained by a trait, written for a type the module that uses it declares.
//!
//! `docs/specs/codegen.md` writes the method into the class of the module that declares the
//! generic, and has it call the instance of the module that declares the type. A type of another
//! module is named with that module in front, so the name alone says which class the call names.

use lumen_ir::{ClassName, Descriptor, Instruction, Lowered, MethodDescriptor, MethodRef};

use crate::common;

/// A module deriving `Eq` for a type of its own, which it settles a constraint of `holder`'s on.
const SETTLING_ITS_OWN: &str = concat!(
    "import holder\n\n",
    "fn is_kept_twice() -> Bool {\n    holder.is_same(Kept(\"a\"), Kept(\"b\"))\n}\n\n",
    "derive Eq for Kept\n\n",
    "type Kept = Kept(String)\n"
);

/// A module settling the same constraint on a type the JVM holds, which has no instance method.
const SETTLING_A_WHOLE_NUMBER: &str =
    "import holder\n\nfn is_one_two() -> Bool {\n    holder.is_same(1, 2)\n}\n";

/// Every instance method the method `named` of the `holder` class calls, in the order it calls
/// them.
fn instances_called(holder: &Lowered, named: &str) -> Vec<MethodRef> {
    let class = common::class_of(holder, &ClassName::new("holder"));
    common::method_of(class, named)
        .body
        .instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::InvokeStatic(reached) if reached.name.contains('$') => {
                Some(reached.clone())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_constrained_generic_written_for_a_type_of_the_asking_module_calls_that_module_s_instance() {
    let holder = common::holder_asked_by(SETTLING_ITS_OWN);

    let called = instances_called(&holder, "is_same$demo$Kept");

    assert_eq!(called.len(), 1);
    assert_eq!(called[0].class, ClassName::new("demo"));
    assert_eq!(called[0].name, "Eq$Kept$is_equal");
    assert_eq!(
        called[0].descriptor,
        MethodDescriptor::new(
            vec![
                Descriptor::reference("demo/Kept"),
                Descriptor::reference("demo/Kept")
            ],
            Some(Descriptor::Boolean)
        )
    );
}

#[test]
fn the_module_settling_the_type_writes_the_instance_method_the_other_one_calls() {
    let lowered = common::lowered_reaching(SETTLING_ITS_OWN, &common::holder());

    assert!(common::has_method(&lowered, "Eq$Kept$is_equal"));
}

#[test]
fn a_constrained_generic_written_for_a_type_the_jvm_holds_calls_no_instance_at_all() {
    let holder = common::holder_asked_by(SETTLING_A_WHOLE_NUMBER);

    assert_eq!(instances_called(&holder, "is_same$Int"), Vec::new());
}

/// A use of `holder.is_same`, the method the set it settles names, and the instance that calls.
///
/// A type the JVM holds has no instance method: what its `Eq` amounts to is written out in place,
/// which `docs/specs/traits.md` states and `demo` never writes a method for.
const CONSTRAINED: [(&str, &str, Option<&str>); 4] = [
    ("1, 2", "is_same$Int", None),
    ("\"a\", \"b\"", "is_same$String", None),
    ("true, false", "is_same$Bool", None),
    (
        "Kept(\"a\"), Kept(\"b\")",
        "is_same$demo$Kept",
        Some("Eq$Kept$is_equal"),
    ),
];

/// `docs/specs/codegen.md`: a method written for a set calls the instance each type in it has.
///
/// The class it calls is the class of the module that declares the type, which is `demo` here
/// however the method itself is written into `holder`.
#[hegel::test]
fn a_method_written_for_a_set_calls_the_instance_each_type_in_that_set_has(tc: hegel::TestCase) {
    let (written, named, instance) = tc.draw(hegel::generators::sampled_from(&CONSTRAINED));
    let source = format!(
        "import holder\n\nfn is_it() -> Bool {{\n    holder.is_same({written})\n}}\n\n\
         derive Eq for Kept\n\ntype Kept = Kept(String)\n"
    );

    let called = instances_called(&common::holder_asked_by(&source), named);

    assert_eq!(
        called
            .iter()
            .map(|reached| (reached.class.written(), reached.name.as_str()))
            .collect::<Vec<(&str, &str)>>(),
        instance.map_or_else(Vec::new, |name| vec![("demo", name)]),
        "`holder` writes `{named}` calling what `{written}` settled"
    );
}

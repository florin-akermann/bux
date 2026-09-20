//! Helpers shared by the lowering's behaviour tests.
//!
//! A behaviour is stated as the Lumen source it is about, run through every phase before this
//! one, because what the lowering says is only ever about a module the compiler has accepted.

use lumen_holes::Whole;
use lumen_ir::{Body, Class, ClassName, Instruction, Lowered, Method, MethodRef, lower};

/// The classes `source` becomes, as a module named `demo`.
pub fn lowered(source: &str) -> Lowered {
    let program = lumen_parser::parse(source).expect("the example parses");
    let resolved = lumen_resolver::resolve(program).expect("every name of the example resolves");
    let typed = lumen_types::check(resolved).expect("every expression of the example has a type");
    let whole = Whole::of_module(&typed).expect("the example holds no hole");
    lower(&whole, "demo")
}

/// The body of the function `name` of the module.
pub fn body_of<'a>(lowered: &'a Lowered, name: &str) -> &'a Body {
    &method_of(class_of(lowered, &ClassName::new("demo")), name).body
}

/// Whether the module class writes a method called `name`.
pub fn has_method(lowered: &Lowered, name: &str) -> bool {
    methods_named(lowered, name) > 0
}

/// How many methods the module class writes under `name`, which is one or none.
pub fn methods_named(lowered: &Lowered, name: &str) -> usize {
    class_of(lowered, &ClassName::new("demo"))
        .methods
        .iter()
        .filter(|method| method.name == name)
        .count()
}

/// The class written as `wanted`, which the example is about.
pub fn class_of<'a>(lowered: &'a Lowered, wanted: &ClassName) -> &'a Class {
    lowered
        .classes
        .iter()
        .find(|class| &class.name == wanted)
        .unwrap_or_else(|| panic!("the module writes a class named {}", wanted.written()))
}

/// The method of `class` written as `name`.
pub fn method_of<'a>(class: &'a Class, name: &str) -> &'a Method {
    class
        .methods
        .iter()
        .find(|method| method.name == name)
        .unwrap_or_else(|| panic!("the class writes a method named {name}"))
}

/// Whether the body calls the method `name` of `class`, however it reaches it.
pub fn calls(body: &Body, class: &ClassName, name: &str) -> bool {
    body.instructions
        .iter()
        .filter_map(called)
        .any(|reference| &reference.class == class && reference.name == name)
}

/// The method an instruction names, when the instruction is a call at all.
pub fn called(instruction: &Instruction) -> Option<&MethodRef> {
    match instruction {
        Instruction::Construct(reference)
        | Instruction::InvokeStatic(reference)
        | Instruction::InvokeVirtual(reference)
        | Instruction::InvokeInterface(reference) => Some(reference),
        _ => None,
    }
}

/// The names of the classes the module writes, in the order it writes them.
pub fn written(lowered: &Lowered) -> Vec<&str> {
    lowered
        .classes
        .iter()
        .map(|class| class.name.written())
        .collect()
}

/// The names of the fields `class` holds, in the order it declares them.
pub fn holds(class: &Class) -> Vec<&str> {
    class
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect()
}

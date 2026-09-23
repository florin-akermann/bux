//! Helpers shared by the class-file writer's behaviour tests.
//!
//! A behaviour is stated as the lowered class it is about, because that is what the writer takes,
//! or as the Lumen source it is about, where the behaviour is one a whole module has.

use lumen_ir::Reached;
use lumen_ir::{
    Body, Class, ClassName, Descriptor, Instruction, Lowered, Method, MethodDescriptor,
};
use lumen_jvm::ClassFile as Written;

use crate::reader::{self, ClassFile};

/// The class files `source` becomes, as a module named `demo`.
pub fn compiled(source: &str) -> Vec<Written> {
    let program = lumen_parser::parse(source).expect("the example parses");
    let resolved =
        lumen_resolver::resolve(program, "demo").expect("every name of the example resolves");
    let typed = lumen_types::check(resolved, &lumen_types::Imported::default())
        .expect("every expression of the example has a type");
    let whole = lumen_holes::Whole::of_module(&typed).expect("the module holds no hole");
    lumen_jvm::write(&[lumen_ir::lower(&whole, "demo", &lumen_ir::Asked::default())])
}

/// The class written to `path`, read back.
pub fn one_of(files: &[Written], path: &str) -> ClassFile {
    let file = files
        .iter()
        .find(|file| file.path == path)
        .unwrap_or_else(|| panic!("the module writes {path}"));
    reader::read(&file.bytes)
}

/// The one class `classes` writes, read back.
pub fn written(class: Class) -> ClassFile {
    let lowered = Lowered {
        asks: lumen_ir::Asked::default(),
        classes: vec![class],
    };
    let files = lumen_jvm::write(&[lowered]);
    assert_eq!(files.len(), 1, "one class is one file");
    reader::read(&files[0].bytes)
}

/// A class named `demo` holding one static method built from `instructions`.
pub fn module_with(name: &str, descriptor: MethodDescriptor, body: Body) -> Class {
    let mut class = Class::new(ClassName::new("demo"));
    class.methods.push(Method {
        name: name.to_owned(),
        descriptor,
        reached: Reached::ThroughTheClass,
        body,
    });
    class
}

/// A body of `instructions` using no locals beyond the parameters.
pub fn body(instructions: Vec<Instruction>) -> Body {
    Body {
        instructions,
        locals: 0,
        guards: Vec::new(),
    }
}

/// The descriptor of a method taking `parameters` and giving back `result`.
pub fn taking(parameters: Vec<Descriptor>, result: Option<Descriptor>) -> MethodDescriptor {
    MethodDescriptor::new(parameters, result)
}

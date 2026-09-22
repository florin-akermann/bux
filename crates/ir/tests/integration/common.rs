//! Helpers shared by the lowering's behaviour tests.
//!
//! A behaviour is stated as the Lumen source it is about, run through every phase before this
//! one, because what the lowering says is only ever about a module the compiler has accepted.

use lumen_holes::Whole;
use lumen_ir::{Asked, Body, Class, ClassName, Instruction, Lowered, Method, MethodRef, lower};
use lumen_types::{Imported, Surface};

/// The classes `source` becomes, as a module named `demo`.
pub fn lowered(source: &str) -> Lowered {
    lowered_reaching(source, &Imported::default())
}

/// The classes `source` becomes, as a module named `demo` reaching what `imported` offers.
pub fn lowered_reaching(source: &str, imported: &Imported) -> Lowered {
    lowered_as(&Written {
        source,
        named: "demo",
        imported,
        asked: &Asked::default(),
    })
}

/// What the module `held` offers, which is a record and an algebraic data type.
pub fn held() -> Imported {
    let source = concat!(
        "fn pending() -> Payment {\n    Pending\n}\n\n",
        "type User = {\n    name: String\n}\n\n",
        "type Payment =\n    | Sent(String)\n    | Pending\n"
    );
    Imported::default().offering("held", surface(source))
}

/// The library module an import of `named` reaches, as a test asks things of it.
pub fn library(named: &'static str) -> LibraryModule {
    LibraryModule { named }
}

/// One module of the library, which is Lumen source the compiler carries rather than a file.
#[derive(Clone, Copy)]
pub struct LibraryModule {
    /// The name an import writes, which is also the class the module is lowered as.
    named: &'static str,
}

impl LibraryModule {
    /// The classes it becomes, lowered under the name it is reached by.
    pub fn lowered(self) -> Lowered {
        lowered_as(&Written {
            source: self.source(),
            named: self.named,
            imported: &self.imports(),
            asked: &Asked::default(),
        })
    }

    /// What it offers, as inference is given what a module importing it reaches.
    pub fn offered(self) -> Imported {
        Imported::default().offering(self.named, self.offers())
    }

    /// Every name a module importing it reaches, which is what it puts out.
    pub fn offers(self) -> Surface {
        surface_reaching(self.source(), &self.imports())
    }

    /// The class it is, which is the class every function of it is a method of.
    pub fn class(self) -> ClassName {
        ClassName::new(self.named)
    }

    /// What the library modules it imports offer it, which is nothing for most of them.
    ///
    /// The library carries every module an import of it names, so this walks the same way the
    /// loader walks: what a module imports is read before that module is.
    fn imports(self) -> Imported {
        imported_by(self.source()).fold(Imported::default(), |imported, module| {
            imported.offering(module, library(module).offers())
        })
    }

    /// The source the compiler carries for it.
    fn source(self) -> &'static str {
        lumen_resolver::library::source_of(self.named).expect("the library carries the module")
    }
}

/// Every module `source` imports, which canonical form writes one to a line of its own.
fn imported_by(source: &'static str) -> impl Iterator<Item = &'static str> {
    source
        .lines()
        .filter_map(|line| line.strip_prefix("import "))
}

/// A module declaring generics, which is what another module reaches one of through an import.
///
/// `held` writes its type parameter out, `passed` writes none and lets inference find it, and
/// `counted` writes one that reaches no further than the inside of a `List`.
pub const HOLDER: &str = concat!(
    "fn held<T>(value: T) -> T {\n    value\n}\n\n",
    "fn passed(value) {\n    value\n}\n\n",
    "fn counted<T>(values: List<T>) -> Int {\n    1\n}\n"
);

/// What [`HOLDER`] offers, under the name `holder`, which is the name a test imports it by.
pub fn holder() -> Imported {
    Imported::default().offering("holder", surface(HOLDER))
}

/// One module a behaviour is about, as the phases before lowering are given it.
pub struct Written<'w> {
    pub source: &'w str,
    /// The name it is lowered under, which is the class its functions are methods of.
    pub named: &'w str,
    /// What the modules it imports offer it.
    pub imported: &'w Imported,
    /// What a module already lowered has asked of it, which is what a generic of it is asked for.
    pub asked: &'w Asked,
}

/// The classes `written` becomes, under the name it says and knowing what it was asked for.
pub fn lowered_as(written: &Written<'_>) -> Lowered {
    let program = lumen_parser::parse(written.source).expect("the example parses");
    let resolved = lumen_resolver::resolve(program).expect("every name of the example resolves");
    let typed = lumen_types::check(resolved, written.imported)
        .expect("every expression of the example has a type");
    let whole = Whole::of_module(&typed).expect("the example holds no hole");
    lower(&whole, written.named, written.asked)
}

/// Every name a module importing `source` reaches, which is what `source` puts out.
fn surface(source: &str) -> Surface {
    surface_reaching(source, &Imported::default())
}

/// The same, of a module that reaches what `imported` offers it.
fn surface_reaching(source: &str, imported: &Imported) -> Surface {
    let program = lumen_parser::parse(source).expect("the offered module parses");
    let resolved = lumen_resolver::resolve(program).expect("the offered module resolves");
    let typed = lumen_types::check(resolved, imported)
        .expect("the offered module has a type for every expression");
    typed.surface().clone()
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

/// Each place a `()` is handed to something that holds a reference, and the function holding it.
///
/// `docs/specs/codegen.md` states that a `()` is carried by nothing, so each of these leaves
/// nothing where a reference is wanted and something has to stand for it.
pub const HOLDING_NOTHING: [(&str, &str); 4] = [
    ("held", "fn held() -> Option<()> {\n    Some(())\n}\n"),
    (
        "saved",
        "fn saved() -> Result<(), String> {\n    Ok(())\n}\n",
    ),
    ("written", "fn written() -> List<()> {\n    [()]\n}\n"),
    (
        "boxed",
        "fn boxed() -> Box<()> {\n    Held(())\n}\n\ntype Box<T> = Held(T)\n",
    ),
];

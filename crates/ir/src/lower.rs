//! A checked module as the classes it becomes.
//!
//! Every function of the module is a static method of one class named after the module, and
//! every type it declares is a class of its own. `docs/specs/codegen.md` states the layout, and
//! the decisions this phase makes — what a value is carried by, where a binding lives, what a
//! `match` tests — are all made here, so that writing the bytes never has to make one.

mod body;
mod classes;
mod equality;
mod escape;
mod expr;
mod generic;
mod literal;
mod modules;
mod operator;
mod pattern;
mod prelude;
mod shape;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use lumen_ast::{Function, InstanceDeclaration, Item, Span};
use lumen_holes::Whole;
use lumen_types::{Type, TypedProgram};

use crate::Lowered;
use crate::class::{Class, Method, Reached};
use crate::code::{Body, Instruction, MethodRef};
use crate::descriptor::{Descriptor, MethodDescriptor};
use crate::lower::body::Builder;
use crate::lower::generic::Instantiation;
use crate::lower::shape::Shapes;

/// What Lumen calls the function a program starts at, and what a JVM calls the method it does.
const START: &str = "main";

/// Lowers `whole` to the classes a JVM loads, as a module of the name `module`.
///
/// A [`Whole`] is a module that holds no hole, which is the only kind there is anything to
/// lower: `docs/specs/holes.md` states what a build does with the other kind.
#[must_use]
pub fn lower(whole: &Whole<'_>, module: &str) -> Lowered {
    let typed = whole.typed();
    let lowering = Lowering {
        typed,
        shapes: Shapes::of(typed.resolved(), module),
        declared: declarations_of(typed),
        answers: instances_of(typed),
        owed: RefCell::new(Vec::new()),
    };
    let module_class = lowering.module_class();
    let reads = modules::reads_a_file(&module_class);
    let mut classes = vec![module_class];
    classes.extend(lowering.shapes.classes());
    if reads {
        classes.push(modules::files_class(&lowering.shapes));
    }
    Lowered { classes }
}

/// What lowering one module holds: the types it was given, and the classes it writes them as.
pub(crate) struct Lowering<'a> {
    pub(crate) typed: &'a TypedProgram,
    pub(crate) shapes: Shapes,
    /// Every function the module writes, by the span of the name declaring it.
    declared: HashMap<Span, Declared<'a>>,
    /// The method each instance writes, by the trait method it answers and the type it is for.
    answers: HashMap<(String, String), Span>,
    /// The methods still owed, which lowering a body adds to as it meets a use of a generic.
    owed: RefCell<Vec<Owed>>,
}

/// One function the module writes: the source of it, and what a JVM calls the method it becomes.
struct Declared<'a> {
    function: &'a Function,
    /// An instance's method is named for its trait and its type; every other is named as written.
    named: String,
}

/// One method the module still owes: a function, and what one use of it settled its types at.
struct Owed {
    declared: Span,
    at: Instantiation,
}

impl Lowering<'_> {
    /// The class the module itself is: the functions it declares, as the methods they become.
    ///
    /// A function that declares no type parameter is one method, written whether anything calls
    /// it or not. A generic is one method per set of types it is used at, so lowering a body is
    /// what asks for the methods that body needs, and the asking goes on until nothing is owed.
    fn module_class(&self) -> Class {
        let mut class = Class::new(self.shapes.module().clone());
        self.owe_every_plain_function();
        class.methods = self.methods_owed();
        class.methods.extend(entry_point(&class));
        class
    }

    /// Asks for the method of every function a use of the module reaches on its own, in order.
    ///
    /// A function that declares no type parameter is written whether anything calls it or not,
    /// because the module declares it and `docs/specs/modules.md` makes that public. `main` is
    /// asked for however it is declared, because running the module is the use it has.
    fn owe_every_plain_function(&self) {
        let reached = self
            .functions()
            .filter(|function| function.type_parameters.is_empty() || function.name.text == START);
        for function in reached {
            self.owe(function.name.span, Instantiation::whole());
        }
    }

    /// Every method the module owes, each written once, until nothing is owed any more.
    ///
    /// Writing one asks for the methods its body reaches, so the list grows as it is drained.
    /// A method is the one already written when it is called the same and takes the same, which
    /// is what a JVM tells apart and what a module class already carries `main` twice under.
    fn methods_owed(&self) -> Vec<Method> {
        let mut written = HashSet::new();
        let mut methods = Vec::new();
        while let Some(owed) = self.next_owed() {
            let declared = &self.declared[&owed.declared];
            let named = owed.at.names(&declared.named);
            let signature = self.signature(owed.declared, &owed.at);
            if written.insert((named.clone(), signature.descriptor())) {
                methods.push(self.method(declared.function, named, &owed.at));
            }
        }
        methods
    }

    /// Every function the module writes, in the order it writes them.
    fn functions(&self) -> impl Iterator<Item = &Function> {
        self.typed.resolved().program().functions()
    }

    /// The method the instance for `at` writes for the trait method `method`, where there is one.
    ///
    /// An instance the compiler supplies has none, which is what says a call of it is written
    /// out where it stands rather than made; `docs/specs/traits.md` states the two cases.
    pub(crate) fn answering(&self, method: &str, at: &lumen_types::Type) -> Option<Span> {
        let Type::Named { name, .. } = at else {
            return None;
        };
        self.answers
            .get(&(method.to_owned(), name.clone()))
            .copied()
    }

    /// The method a use of `name` reaches: what it is called, and what it takes and gives back.
    ///
    /// Asking is what has the method written. A use inside a generic body is a use at the types
    /// the body was written for, so what that body settled is applied before the use is read.
    pub(crate) fn used(&self, declared: Span, at: Span, within: &Instantiation) -> Reaching {
        let function = self.declared[&declared].function;
        if function.type_parameters.is_empty() {
            return Reaching {
                named: self.declared[&declared].named.clone(),
                signature: self.signature(declared, &Instantiation::whole()),
            };
        }
        let used = within.substituted(self.used_as(at));
        let settled = Instantiation::of(function, self.used_as(declared), &used);
        let named = settled.names(&self.declared[&declared].named);
        let signature = self.signature(declared, &settled);
        self.owe(declared, settled);
        Reaching { named, signature }
    }

    fn method(&self, function: &Function, named: String, at: &Instantiation) -> Method {
        let signature = self.signature(function.name.span, at);
        let mut builder = Builder::entering(self, function, &signature, at.clone());
        builder.body(&function.body);
        Method {
            name: named,
            descriptor: signature.descriptor(),
            reached: Reached::ThroughTheClass,
            body: builder.finish(),
        }
    }

    fn owe(&self, declared: Span, at: Instantiation) {
        self.owed.borrow_mut().push(Owed { declared, at });
    }

    /// The next method owed, taken in the order it was asked for so the class is written once.
    ///
    /// Lowering a body asks for the methods that body needs, so the list grows while it is
    /// being drained, and the borrow is let go of before anything is lowered.
    fn next_owed(&self) -> Option<Owed> {
        let mut owed = self.owed.borrow_mut();
        (!owed.is_empty()).then(|| owed.remove(0))
    }

    /// What a function reached through another module takes and gives back, read off its use.
    ///
    /// The module declaring it has written it once, under the name it is declared with, because
    /// `docs/specs/modules.md` offers no generic through an import.
    pub(crate) fn reached_through(&self, used: Span) -> Signature {
        self.signature(used, &Instantiation::whole())
    }

    /// What the function declared at `declared` takes and gives back, at the types `at` settled.
    fn signature(&self, declared: Span, at: &Instantiation) -> Signature {
        let Type::Function { parameters, result } = self.used_as(declared) else {
            unreachable!("a function is declared with a function type")
        };
        Signature {
            parameters: parameters
                .iter()
                .map(|of| self.shapes.carried(&at.substituted(of)))
                .collect(),
            result: self.shapes.carried(&at.substituted(result)),
        }
    }

    /// The type of whatever is written at `written`, which every function and use has.
    fn used_as(&self, written: Span) -> &Type {
        self.typed
            .type_of(written)
            .expect("inference gave every function and every use of one a type")
    }
}

/// The method one use reaches, which is what a call is written with.
pub(crate) struct Reaching {
    pub(crate) named: String,
    pub(crate) signature: Signature,
}

/// Every function the module writes, by the span of the name declaring it.
///
/// An instance's method is one of them: it has a body like any other function's, and what tells
/// it apart is the name a JVM reaches it by, which is its trait's and its type's as well as its
/// own. `docs/specs/traits.md` states the name, and `$` is what joins the three.
fn declarations_of(typed: &TypedProgram) -> HashMap<Span, Declared<'_>> {
    let mut declared = HashMap::new();
    for item in &typed.resolved().program().items {
        match item {
            Item::Function(function) => {
                declared.insert(function.name.span, as_written(function));
            }
            Item::Instance(instance) => {
                declared.extend(
                    instance
                        .methods
                        .iter()
                        .map(|method| (method.name.span, as_an_instance(instance, method))),
                );
            }
            Item::Import(_) | Item::Type(_) | Item::Trait(_) => {}
        }
    }
    declared
}

/// A function the module declares, which a JVM reaches by the name it is written with.
fn as_written(function: &Function) -> Declared<'_> {
    Declared {
        function,
        named: function.name.text.clone(),
    }
}

/// One method of an instance, which a JVM reaches by its trait, its type, and its own name.
fn as_an_instance<'a>(instance: &InstanceDeclaration, method: &'a Function) -> Declared<'a> {
    let named = format!(
        "{}${}${}",
        instance.trait_name.text, instance.for_type.text, method.name.text
    );
    Declared {
        function: method,
        named,
    }
}

/// The method each instance writes, by the trait method it answers and the type it is for.
///
/// One trait and one type have one instance, which name resolution has already held the module
/// to, so what a trait method at a type reaches is settled by looking here and nowhere else.
fn instances_of(typed: &TypedProgram) -> HashMap<(String, String), Span> {
    let mut answers = HashMap::new();
    for item in &typed.resolved().program().items {
        let Item::Instance(instance) = item else {
            continue;
        };
        for method in &instance.methods {
            let answering = (method.name.text.clone(), instance.for_type.text.clone());
            answers.insert(answering, method.name.span);
        }
    }
    answers
}

/// What a function takes and gives back, with a place for each parameter the source wrote.
pub(crate) struct Signature {
    /// One per parameter written, which is carried by nothing when its type is `()`.
    pub(crate) parameters: Vec<Option<Descriptor>>,
    pub(crate) result: Option<Descriptor>,
}

impl Signature {
    /// The descriptor of the method, which leaves out every parameter carried by nothing.
    pub(crate) fn descriptor(&self) -> MethodDescriptor {
        let taken = self.parameters.iter().flatten().cloned().collect();
        MethodDescriptor::new(taken, self.result.clone())
    }
}

/// Whether `lowered` is a program, which is a module a JVM can be started on.
///
/// A module that declares `main` is written with the entry point `java` looks for; one that
/// does not is a library, and there is nothing to start it at.
#[must_use]
pub fn is_a_program(lowered: &Lowered) -> bool {
    lowered.classes.first().is_some_and(|class| {
        class
            .methods
            .iter()
            .any(|method| method.name == START && method.descriptor.parameters == [arguments()])
    })
}

/// What the JVM starts at, which a module declaring `main` is written with and no other is.
///
/// The entry point is not a function anyone wrote: it is the shape `java` looks for, and all it
/// does is call the `main` the module declares. Writing it with the module means running the
/// module class directly is running the program, so `lumen run` supplies nothing of its own.
fn entry_point(class: &Class) -> Option<Method> {
    let started = MethodDescriptor::new(Vec::new(), None);
    let declared = class
        .methods
        .iter()
        .any(|method| method.name == START && method.descriptor == started);
    if !declared {
        return None;
    }
    Some(Method {
        name: START.to_owned(),
        descriptor: MethodDescriptor::new(vec![arguments()], None),
        reached: Reached::ThroughTheClass,
        body: Body {
            instructions: vec![
                Instruction::InvokeStatic(MethodRef {
                    class: class.name.clone(),
                    name: START.to_owned(),
                    descriptor: started,
                }),
                Instruction::Return(None),
            ],
            locals: 0,
            guards: Vec::new(),
        },
    })
}

/// What a JVM hands the entry point, which is the one thing Lumen never looks at.
fn arguments() -> Descriptor {
    Descriptor::array(Descriptor::reference("java/lang/String"))
}

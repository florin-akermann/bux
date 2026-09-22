//! A checked module as the classes it becomes.
//!
//! Every function of the module is a static method of one class named after the module, and
//! every type it declares is a class of its own. `docs/specs/codegen.md` states the layout, and
//! the decisions this phase makes — what a value is carried by, where a binding lives, what a
//! `match` tests — are all made here, so that writing the bytes never has to make one.

mod body;
mod classes;
mod derive;
mod escape;
mod expr;
mod functions;
mod generic;
mod lists;
mod literal;
mod modules;
mod operator;
mod pattern;
mod reaching;
mod shape;
mod standard;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use lumen_ast::{DeriveDeclaration, ExternDeclaration, Function, InstanceDeclaration};
use lumen_ast::{Item, Name};
use lumen_ast::{Span, TypeDeclaration};
use lumen_holes::Whole;
use lumen_resolver::prelude;
use lumen_types::{Type, TypedProgram};

use crate::Lowered;
use crate::asked::{Asked, Specialisation};
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
///
/// `asked` is every method a module already lowered has asked of this one, which is how a
/// generic reached through an import comes to be written: `docs/specs/codegen.md` has the module
/// declaring it write the method, so a build lowers a module after everything that imports it.
#[must_use]
pub fn lower(whole: &Whole<'_>, module: &str, asked: &Asked) -> Lowered {
    let typed = whole.typed();
    let lowering = Lowering {
        typed,
        module: module.to_owned(),
        shapes: Shapes::of(typed.resolved(), module, typed.reached()),
        declared: declarations_of(typed),
        answers: instances_of(typed),
        owed: RefCell::new(Vec::new()),
        asks: RefCell::new(Asked::default()),
    };
    let mut classes = vec![lowering.module_class(asked)];
    classes.extend(lowering.shapes.classes());
    Lowered {
        classes,
        asks: lowering.asks.into_inner(),
    }
}

/// What lowering one module holds: the types it was given, and the classes it writes them as.
pub(crate) struct Lowering<'a> {
    pub(crate) typed: &'a TypedProgram,
    /// The name this module is reached by, which is what a type of its own is asked for under.
    module: String,
    pub(crate) shapes: Shapes,
    /// Every function the module writes, by the span of the name declaring it.
    declared: HashMap<Span, Declared<'a>>,
    /// The method each instance writes, by the trait method it answers and the type it is for.
    answers: HashMap<(String, String), Span>,
    /// The methods still owed, which lowering a body adds to as it meets a use of a generic.
    owed: RefCell<Vec<Owed>>,
    /// The methods this module has asked other modules for, which their own builds write.
    asks: RefCell<Asked>,
}

/// One method the module writes: where its body comes from, and what a JVM calls it.
struct Declared<'a> {
    /// An instance's method is named for its trait and its type; every other is named as written.
    named: String,
    body: Written<'a>,
}

/// Where the body of one method comes from.
///
/// A derive writes a method no source function stands behind, which `docs/specs/derive.md`
/// states, and what it does follows from the declaration the derive names instead.
enum Written<'a> {
    Source(&'a Function),
    /// The Java member an `extern` names, which `docs/specs/interop.md` states the body of.
    Reaching(&'a ExternDeclaration),
    /// The trait the derive names, and the declaration the body follows from.
    Derived {
        of: String,
        declaration: &'a TypeDeclaration,
    },
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
    fn module_class(&self, asked: &Asked) -> Class {
        let mut class = Class::new(self.shapes.module().clone());
        self.owe_every_method_nothing_has_to_ask_for();
        self.owe_every_method_another_module_asked_for(asked);
        class.methods = self.methods_owed();
        class.methods.extend(entry_point(&class));
        class
    }

    /// Asks for the method of every function a use of the module reaches on its own, in order.
    ///
    /// A function that declares no type parameter is written whether anything calls it or not,
    /// because the module declares it and `docs/specs/modules.md` makes that public. `main` is
    /// asked for however it is declared, because running the module is the use it has.
    fn owe_every_method_nothing_has_to_ask_for(&self) {
        let reached = self
            .functions()
            .filter(|function| function.type_parameters.is_empty() || function.name.text == START);
        for function in reached {
            self.owe(function.name.span, Instantiation::whole());
        }
        for (named, _) in derived_in(self.typed) {
            self.owe(named.span, Instantiation::whole());
        }
        for declaration in self.typed.resolved().program().externs() {
            self.owe(declaration.name.span, Instantiation::whole());
        }
    }

    /// Asks for the method of every set of types a use in another module settled one of these
    /// generics at.
    ///
    /// A use settles its types where it is written and the method is written here, which
    /// `docs/specs/codegen.md` states, so this is the only way such a method is asked for. A set
    /// asked for a function this module does not declare is nothing: name resolution and
    /// inference have both already held the asking module to what this one offers.
    fn owe_every_method_another_module_asked_for(&self, asked: &Asked) {
        for one in asked.of_module(&self.module) {
            let Some(function) = self.declares(one.function()) else {
                continue;
            };
            self.owe(
                function.name.span,
                Instantiation::asked_for(function, one.settled()),
            );
        }
    }

    /// The function this module declares as `named`, where it declares one.
    fn declares(&self, named: &str) -> Option<&Function> {
        self.functions()
            .find(|function| function.name.text == named)
    }

    /// Asks `module` for the method of `function` at the types `settled`, and says what it is
    /// called.
    ///
    /// The name is the one the other module's build writes the method under, arrived at from the
    /// types alone, which is what has the two agree without either reading the other's tree.
    pub(crate) fn asking(&self, module: &str, function: &str, settled: Vec<Type>) -> String {
        let named = generic::names(function, &settled);
        self.asks
            .borrow_mut()
            .note(Specialisation::of(module, function, settled));
        named
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
                methods.push(self.method(&owed, named, &signature));
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
    /// An instance over a type the JVM holds has none, which is what says a call of it is written
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
        let Written::Source(function) = self.declared[&declared].body else {
            return self.plainly(declared);
        };
        if function.type_parameters.is_empty() {
            return self.plainly(declared);
        }
        let used = within.substituted(self.used_as(at));
        let settled = Instantiation::of(function, self.used_as(declared), &used);
        let named = settled.names(&self.declared[&declared].named);
        let signature = self.signature(declared, &settled);
        self.owe(declared, settled);
        Reaching { named, signature }
    }

    /// The method a use of a function that declares no type parameter reaches.
    pub(crate) fn plainly(&self, declared: Span) -> Reaching {
        Reaching {
            named: self.declared[&declared].named.clone(),
            signature: self.signature(declared, &Instantiation::whole()),
        }
    }

    fn method(&self, owed: &Owed, named: String, signature: &Signature) -> Method {
        Method {
            name: named,
            descriptor: signature.descriptor(),
            reached: Reached::ThroughTheClass,
            body: self.written_body(owed, signature),
        }
    }

    /// What one method does: the body a function writes, or the one a derive follows from.
    fn written_body(&self, owed: &Owed, signature: &Signature) -> Body {
        match &self.declared[&owed.declared].body {
            Written::Source(function) => {
                let mut builder = Builder::entering(self, function, signature, owed.at.clone());
                builder.body(&function.body);
                builder.finish()
            }
            Written::Derived { of, declaration } => derive::body(self, of, declaration),
            Written::Reaching(declaration) => reaching::body(self, declaration, signature),
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
    /// The module declaring it has written it once, under the name it is declared with, which is
    /// every function of it that declares no type parameter and settles no type.
    pub(crate) fn reached_through(&self, used: Span) -> Signature {
        self.signature(used, &Instantiation::whole())
    }

    /// What a method written for one set of types takes and gives back, given the type it has.
    ///
    /// This is a generic of another module, whose method that module writes: the type is the one
    /// the declaration has once that set is settled, rather than the one the use has.
    pub(crate) fn written_as(&self, declared: &Type) -> Signature {
        let Type::Function { parameters, result } = declared else {
            unreachable!("a function is declared with a function type")
        };
        Signature {
            parameters: parameters
                .iter()
                .map(|of| self.shapes.carried(of))
                .collect(),
            result: self.shapes.carried(result),
        }
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
            Item::Derive(derive) => {
                for named in &derive.traits {
                    declared.insert(named.span, as_derived(typed, derive, named));
                }
            }
            Item::Extern(declaration) => {
                declared.insert(declaration.name.span, as_reaching(declaration));
            }
            Item::Import(_) | Item::Type(_) | Item::Trait(_) => {}
        }
    }
    declared
}

/// The one method a derive of one trait writes, which a JVM reaches as a written instance's.
///
/// `docs/specs/derive.md` makes each of the four standard traits one method, so a derive naming
/// two traits writes two methods and name resolution has already refused a derive of anything
/// with more or fewer.
fn as_derived<'a>(typed: &'a TypedProgram, derive: &DeriveDeclaration, of: &Name) -> Declared<'a> {
    let method = prelude::method_of(&of.text).expect("a derivable trait declares one method");
    Declared {
        named: format!("{}${}${method}", of.text, derive.for_type.text),
        body: Written::Derived {
            of: of.text.clone(),
            declaration: declared_as(typed, &derive.for_type.text),
        },
    }
}

/// The declaration of the type called `named`, which is what a derive of it follows from.
fn declared_as<'a>(typed: &'a TypedProgram, named: &str) -> &'a TypeDeclaration {
    typed
        .resolved()
        .program()
        .items
        .iter()
        .find_map(|item| match item {
            Item::Type(declaration) if declaration.name.text == named => Some(declaration),
            _ => None,
        })
        .expect("name resolution gave the derive's type a declaration in this module")
}

/// The one method an `extern` writes, which a JVM reaches as it reaches any other function's.
fn as_reaching(declaration: &ExternDeclaration) -> Declared<'_> {
    Declared {
        named: declaration.name.text.clone(),
        body: Written::Reaching(declaration),
    }
}

/// A function the module declares, which a JVM reaches by the name it is written with.
fn as_written(function: &Function) -> Declared<'_> {
    Declared {
        named: function.name.text.clone(),
        body: Written::Source(function),
    }
}

/// One method of an instance, which a JVM reaches by its trait, its type, and its own name.
fn as_an_instance<'a>(instance: &InstanceDeclaration, method: &'a Function) -> Declared<'a> {
    let named = format!(
        "{}${}${}",
        instance.trait_name.text, instance.for_type.text, method.name.text
    );
    Declared {
        named,
        body: Written::Source(method),
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
    for (named, derive) in derived_in(typed) {
        let method = prelude::method_of(&named.text).expect("a derivable trait declares one");
        let answering = (method.to_owned(), derive.for_type.text.clone());
        answers.insert(answering, named.span);
    }
    answers
}

/// Every instance a derive writes, as the trait it names and the derive that names it.
fn derived_in(typed: &TypedProgram) -> impl Iterator<Item = (&Name, &DeriveDeclaration)> {
    typed
        .resolved()
        .program()
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Derive(derive) => Some(derive),
            _ => None,
        })
        .flat_map(|derive| derive.traits.iter().map(move |named| (named, derive)))
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

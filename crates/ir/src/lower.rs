//! A checked module as the classes it becomes.
//!
//! Every function of the module is a static method of one class named after the module, and
//! every type it declares is a class of its own. `docs/specs/codegen.md` states the layout, and
//! the decisions this phase makes — what a value is carried by, where a binding lives, what a
//! `match` tests — are all made here, so that writing the bytes never has to make one.

mod body;
mod classes;
mod derive;
mod elements;
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
use lumen_types::{Type, TypeParameter, TypedProgram};

use crate::Lowered;
use crate::asked::{Asked, Asking, Specialisation};
use crate::class::{Class, Method, Reached};
use crate::code::{Body, Instruction, MethodRef};
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
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
    answers: HashMap<(String, String), Answers>,
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

/// The method one instance of this module writes: where it is written, and what it is over.
struct Answers {
    /// The names the instance writes as the arguments of its type, in the order it writes them.
    arguments: Vec<String>,
    declared: Span,
}

/// One method the module still owes, which is a function of its own or an instance over a list.
enum Owed {
    /// A function this module writes: what a JVM calls it, and what one use settled its types at.
    ///
    /// The name is worked out where the method is asked for, because the use that asks for it is
    /// what reaches it: a method and the call of it are named once, in one place, or the two drift.
    Written {
        declared: Span,
        named: String,
        at: Instantiation,
    },
    /// An instance of a standard trait over `List`, at the one type that list holds.
    OverAList(OverAList),
}

/// One method the prelude's instance over `List` writes, at the type the list holds.
///
/// `List` is the compiler's type, so no module declares it and no module's class is its own.
/// The method is written into the class of the module that uses it, exactly as what the prelude's
/// other instances amount to is written where they are used; `docs/specs/codegen.md` states it.
pub(crate) struct OverAList {
    /// The trait it answers, which is one of the four the prelude writes an instance of.
    of: String,
    /// The type the list holds, which every element is handed to the instance of.
    element: Type,
    named: String,
    signature: Signature,
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
        class.methods.extend(self.entry_point(&class));
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
            self.owe_plainly(function.name.span);
        }
        for (named, _) in derived_in(self.typed) {
            self.owe_plainly(named.span);
        }
        for declaration in self.typed.resolved().program().externs() {
            self.owe_plainly(declaration.name.span);
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
            let declared = function.name.span;
            let at = Instantiation::asked_for(function, one.settled());
            let named = at.names(&self.declared[&declared].named);
            self.owe(declared, named, at);
        }
    }

    /// What the JVM starts at, which a module declaring `main` is written with and no other is.
    ///
    /// The entry point is not a function anyone wrote: it is the shape `java` looks for. It
    /// gathers the words the program was run with into the list `main` takes, calls `main`, and
    /// ends the run with the status `main` gave back. Writing it with the module means running
    /// the module class directly is running the program, so `lumen run` supplies nothing of its
    /// own.
    fn entry_point(&self, class: &Class) -> Option<Method> {
        let started = self.started()?;
        Some(Method {
            name: START.to_owned(),
            descriptor: MethodDescriptor::new(vec![arguments()], None),
            reached: Reached::ThroughTheClass,
            body: Body {
                instructions: vec![
                    Instruction::Load {
                        slot: 0,
                        of: arguments(),
                    },
                    Instruction::CollectList,
                    Instruction::InvokeStatic(MethodRef {
                        class: class.name.clone(),
                        name: START.to_owned(),
                        descriptor: started,
                    }),
                    Instruction::LowEightBits,
                    Instruction::InvokeStatic(ended()),
                    Instruction::Return(None),
                ],
                locals: 0,
                guards: Vec::new(),
            },
        })
    }

    /// The method `main` is written as, where the module declares `main` at the one shape.
    ///
    /// The whole signature is read, so a module declaring `main` at another shape is a library,
    /// exactly as one declaring no `main` is.
    fn started(&self) -> Option<MethodDescriptor> {
        let function = self.declares(START)?;
        if *self.used_as(function.name.span) != what_a_program_starts_at() {
            return None;
        }
        Some(
            self.signature(function.name.span, &Instantiation::whole())
                .descriptor(),
        )
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
    pub(crate) fn asking(&self, module: &str, function: &str, asked: Asking) -> String {
        let Asking {
            settled,
            constrained,
        } = asked;
        let named = generic::names(function, &settled, &constrained);
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
            let (named, signature) = self.reached_as(&owed);
            if written.insert((named.clone(), signature.descriptor())) {
                methods.push(self.method(&owed, named, &signature));
            }
        }
        methods
    }

    /// What a JVM calls one method owed, and what that method takes and gives back.
    fn reached_as(&self, owed: &Owed) -> (String, Signature) {
        match owed {
            Owed::Written {
                declared,
                named,
                at,
            } => (named.clone(), self.signature(*declared, at)),
            Owed::OverAList(over) => (over.named.clone(), over.signature.clone()),
        }
    }

    /// Every function the module writes, in the order it writes them.
    fn functions(&self) -> impl Iterator<Item = &Function> {
        self.typed.resolved().program().functions()
    }

    /// The method the instance for `at` writes for the trait method `method`, where there is one.
    ///
    /// An instance over a type the JVM holds has none, which is what says a call of it is written
    /// out where it stands rather than made; `docs/specs/traits.md` states the two cases.
    ///
    /// A type another module declares is named with that module in front, so its instance is
    /// that module's and the call is written against that module's class. Nothing of that
    /// module's tree is read: the method is named for its trait, its type, and itself, exactly as
    /// that module names its own, and what it takes and gives back is the trait method's own
    /// signature at that type. That is how a generic written here for a type the program declares
    /// reaches the program's own instance, which `docs/specs/codegen.md` states.
    pub(crate) fn answering(&self, method: &str, at: &Type) -> Option<Instance> {
        let Type::Named { name, arguments } = at else {
            return None;
        };
        if name == prelude::LIST_TYPE {
            return Some(self.over_a_list(method, at, arguments));
        }
        let Some((module, declared)) = name.split_once('.') else {
            return self.written_here(method, at, name);
        };
        let of = prelude::trait_of(method)
            .expect("a trait two modules both name is one the prelude declares");
        Some(Instance {
            class: ClassName::new(module),
            reaching: Reaching {
                named: generic::names_wholly(&format!("{of}${declared}${method}"), arguments),
                signature: self.written_as(&lumen_types::instance_signature(of, method, at)),
            },
        })
    }

    /// The method the prelude's instance over `List` writes, at the type this list holds.
    ///
    /// It is named for the whole type rather than for its head alone: two lists holding
    /// different types hand their elements to different instances, so each needs a method of its
    /// own. `docs/specs/codegen.md` states the name, and `docs/specs/traits.md` the instance.
    fn over_a_list(&self, method: &str, at: &Type, arguments: &[Type]) -> Instance {
        let of = prelude::trait_of(method)
            .expect("a trait an instance over a list answers is one the prelude declares");
        let named = generic::names_wholly(&over_a_list_named(of, method), arguments);
        let signature = self.written_as(&lumen_types::instance_signature(of, method, at));
        let [element] = arguments else {
            unreachable!("a list is written with the one type it holds")
        };
        self.owed.borrow_mut().push(Owed::OverAList(OverAList {
            of: of.to_owned(),
            element: element.clone(),
            named: named.clone(),
            signature: signature.clone(),
        }));
        Instance {
            class: self.shapes.module().clone(),
            reaching: Reaching { named, signature },
        }
    }

    /// The instance this module writes for a type of its own, where it writes one at all.
    fn written_here(&self, method: &str, at: &Type, named: &str) -> Option<Instance> {
        let answers = self.answers.get(&(method.to_owned(), named.to_owned()))?;
        Some(Instance {
            class: self.shapes.module().clone(),
            reaching: self.instance_method(answers, at),
        })
    }

    /// The method the instance writes, at the types this use of it settled its parameters on.
    ///
    /// An instance over a type written with arguments is generic in them, exactly as a function
    /// declaring type parameters is, so it is written once per set of types it is used at.
    /// `docs/specs/traits.md` states the instance, and `docs/specs/codegen.md` the name.
    fn instance_method(&self, answers: &Answers, at: &Type) -> Reaching {
        let declared = answers.declared;
        let Written::Source(function) = self.declared[&declared].body else {
            return self.plainly(declared);
        };
        if function.type_parameters.is_empty() {
            return self.plainly(declared);
        }
        let Type::Named { arguments, .. } = at else {
            unreachable!("an instance is for a type written by name, which this use settled")
        };
        let settled = Instantiation::asked_for(function, &settled_by(function, answers, arguments));
        let named = generic::names_wholly(&self.declared[&declared].named, arguments);
        let signature = self.signature(declared, &settled);
        self.owe(declared, named.clone(), settled);
        Reaching { named, signature }
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
        self.owe(declared, named.clone(), settled);
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

    /// What one method does: the body a function writes, or the one the compiler writes for it.
    fn written_body(&self, owed: &Owed, signature: &Signature) -> Body {
        match owed {
            Owed::OverAList(over) => elements::body(self, over, signature),
            Owed::Written { declared, at, .. } => self.body_written_for(*declared, at, signature),
        }
    }

    /// The body one function writes, at the types one use of it settled.
    fn body_written_for(&self, declared: Span, at: &Instantiation, signature: &Signature) -> Body {
        match &self.declared[&declared].body {
            Written::Source(function) => {
                let mut builder = Builder::entering(self, function, signature, at.clone());
                builder.body(&function.body);
                builder.finish()
            }
            Written::Derived { of, declaration } => derive::body(self, of, declaration),
            Written::Reaching(declaration) => reaching::body(self, declaration, signature),
        }
    }

    /// Owes the method a function declaring no type parameter writes, under its own name.
    fn owe_plainly(&self, declared: Span) {
        self.owe(
            declared,
            self.declared[&declared].named.clone(),
            Instantiation::whole(),
        );
    }

    fn owe(&self, declared: Span, named: String, at: Instantiation) {
        self.owed.borrow_mut().push(Owed::Written {
            declared,
            named,
            at,
        });
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

/// The method one instance writes, and the class it is written into.
///
/// An instance of a type this module declares is a method of this module's own class, and one of
/// a type another module declares is a method of that module's class. Nothing calling one reads
/// which of the two it is: it asks for the call and is handed it.
pub(crate) struct Instance {
    class: ClassName,
    reaching: Reaching,
}

impl Instance {
    /// The call that reaches it, which is a static call like every other function's.
    pub(crate) fn called(&self) -> Instruction {
        Instruction::InvokeStatic(MethodRef {
            class: self.class.clone(),
            name: self.reaching.named.clone(),
            descriptor: self.reaching.signature.descriptor(),
        })
    }

    /// What it takes, one place per parameter, and what it gives back.
    pub(crate) const fn signature(&self) -> &Signature {
        &self.reaching.signature
    }
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
fn instances_of(typed: &TypedProgram) -> HashMap<(String, String), Answers> {
    let mut answers = HashMap::new();
    for item in &typed.resolved().program().items {
        let Item::Instance(instance) = item else {
            continue;
        };
        for method in &instance.methods {
            let answering = (method.name.text.clone(), instance.for_type.text.clone());
            answers.insert(
                answering,
                written_over(written_as(instance), method.name.span),
            );
        }
    }
    for (named, derive) in derived_in(typed) {
        let method = prelude::method_of(&named.text).expect("a derivable trait declares one");
        let answering = (method.to_owned(), derive.for_type.text.clone());
        answers.insert(answering, written_over(Vec::new(), named.span));
    }
    answers
}

/// One instance over `arguments`, whose method is written where `declared` says.
fn written_over(arguments: Vec<String>, declared: Span) -> Answers {
    Answers {
        arguments,
        declared,
    }
}

/// What this use settled each type parameter the instance declares, in the order it declares them.
fn settled_by(function: &Function, answers: &Answers, at: &[Type]) -> Vec<Type> {
    function
        .type_parameters
        .iter()
        .map(|declared| settling(&declared.name, answers, at))
        .collect()
}

/// What one type parameter the instance declares settled on, which is the argument it is written
/// as.
///
/// A parameter the instance declares and writes as no argument of its type settles on nothing and
/// stands for itself, exactly as one a function's signature never writes does.
fn settling(declared: &Name, answers: &Answers, at: &[Type]) -> Type {
    answers
        .arguments
        .iter()
        .position(|written| *written == declared.text)
        .and_then(|position| at.get(position).cloned())
        .unwrap_or_else(|| Type::Parameter(TypeParameter::written(declared)))
}

/// The names one instance writes as the arguments of the type it is for.
fn written_as(instance: &InstanceDeclaration) -> Vec<String> {
    instance
        .arguments
        .iter()
        .map(|written| written.text.clone())
        .collect()
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
#[derive(Clone)]
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

/// What a JVM calls the method the instance of `of` over a list writes, before its arguments.
fn over_a_list_named(of: &str, method: &str) -> String {
    format!("{of}${}${method}", prelude::LIST_TYPE)
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

/// The one shape a program starts at, written the way an author writes it.
///
/// A refusal names this, so an author reads what to write rather than that what they wrote is
/// wrong. `what_a_program_starts_at` is the same shape as a type, and a test reads the two
/// together so neither moves without the other.
pub const THE_ONE_SHAPE: &str = "fn main(arguments: List<String>) -> Int";

/// The one shape a program starts at, which `docs/design.md` section 11 states.
///
/// A program is run with the words written after its file and ends with a status, so `main`
/// takes the one and gives back the other.
fn what_a_program_starts_at() -> Type {
    Type::function(vec![Type::list(Type::string())], Type::int())
}

/// What a JVM hands the entry point, which is the one thing Lumen never looks at.
fn arguments() -> Descriptor {
    Descriptor::array(Descriptor::reference("java/lang/String"))
}

/// The method a JVM ends a run through, which is where a program's status reaches the system.
fn ended() -> MethodRef {
    MethodRef {
        class: ClassName::new("java/lang/System"),
        name: "exit".to_owned(),
        descriptor: MethodDescriptor::new(vec![Descriptor::Integer], None),
    }
}

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
mod pattern;
mod prelude;
mod shape;

use lumen_ast::{Function, Item, Span};
use lumen_holes::Whole;
use lumen_types::{Type, TypedProgram};

use crate::Lowered;
use crate::class::{Class, Method, Reached};
use crate::code::{Body, Instruction, MethodRef};
use crate::descriptor::{Descriptor, MethodDescriptor};
use crate::lower::body::Builder;
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
    };
    let mut classes = vec![lowering.module_class()];
    classes.extend(lowering.shapes.classes());
    Lowered { classes }
}

/// What lowering one module holds: the types it was given, and the classes it writes them as.
pub(crate) struct Lowering<'a> {
    pub(crate) typed: &'a TypedProgram,
    pub(crate) shapes: Shapes,
}

impl Lowering<'_> {
    /// The class the module itself is: one static method per function, in the order written.
    fn module_class(&self) -> Class {
        let mut class = Class::new(self.shapes.module().clone());
        for item in &self.typed.resolved().program().items {
            if let Item::Function(function) = item {
                class.methods.push(self.method(function));
            }
        }
        class.methods.extend(entry_point(&class));
        class
    }

    fn method(&self, function: &Function) -> Method {
        let signature = self.signature(function.name.span);
        let mut builder = Builder::entering(self, function, &signature);
        builder.body(&function.body);
        Method {
            name: function.name.text.clone(),
            descriptor: signature.descriptor(),
            reached: Reached::ThroughTheClass,
            body: builder.finish(),
        }
    }

    /// What the function declared at `declared` takes and gives back.
    pub(crate) fn signature(&self, declared: Span) -> Signature {
        let Some(Type::Function { parameters, result }) = self.typed.type_of(declared) else {
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

    /// What the type of whatever is written at `written` is carried by.
    pub(crate) fn carried(&self, written: Span) -> Option<Descriptor> {
        self.typed
            .type_of(written)
            .and_then(|of| self.shapes.carried(of))
    }
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
        },
    })
}

/// What a JVM hands the entry point, which is the one thing Lumen never looks at.
fn arguments() -> Descriptor {
    Descriptor::array(Descriptor::reference("java/lang/String"))
}

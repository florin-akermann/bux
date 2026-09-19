//! A checked module as the classes it becomes.
//!
//! Every function of the module is a static method of one class named after the module, and
//! every type it declares is a class of its own. `docs/specs/codegen.md` states the layout, and
//! the decisions this phase makes — what a value is carried by, where a binding lives, what a
//! `match` tests — are all made here, so that writing the bytes never has to make one.

mod body;
mod classes;
mod equality;
mod expr;
mod pattern;
mod shape;

use lumen_ast::{Function, Item, Span};
use lumen_types::{Type, TypedProgram};

use crate::Lowered;
use crate::class::{Class, Method, Reached};
use crate::descriptor::{Descriptor, MethodDescriptor};
use crate::lower::body::Builder;
use crate::lower::shape::Shapes;

/// Lowers `typed` to the classes a JVM loads, as a module of the name `module`.
#[must_use]
pub fn lower(typed: &TypedProgram, module: &str) -> Lowered {
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

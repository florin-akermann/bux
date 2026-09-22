//! What a call of a name inside a module runs, which is a static method of that module's class.
//!
//! A module loaded from a file is a class of its own, so a call reaching into one is the same
//! call as any other, written against the other class. The library is loaded from the source the
//! compiler carries, which `docs/specs/library.md` states, so `io` and `files` are classes like
//! any other and a call of one is written like any other.

use lumen_ast::{Expr, Name, Span};
use lumen_resolver::prelude;
use lumen_types::Type;

use crate::asked::Asking;
use crate::code::{Instruction, MethodRef};
use crate::descriptor::{ClassName, Descriptor};
use crate::lower::Signature;
use crate::lower::body::Builder;

/// A name a call reaches through a module, and where the call writes it.
///
/// `used` is where the name is reached, which is what the types of the call are read off.
pub(crate) struct Through<'w> {
    pub(crate) module: &'w Name,
    pub(crate) name: &'w Name,
    pub(crate) used: Span,
}

impl Builder<'_> {
    /// A call of `module.name`, which inference has already settled the meaning of.
    pub(crate) fn inside_module(
        &mut self,
        reached: &Through<'_>,
        arguments: &[&Expr],
    ) -> Option<Descriptor> {
        if let Some(shape) = self
            .lowering
            .shapes
            .offered(&reached.module.text, &reached.name.text)
            .cloned()
        {
            return Some(self.builds(&shape, arguments));
        }
        if let Some(held) = self.held_by_the_compiler(reached, arguments) {
            return Some(held);
        }
        self.in_another_module(reached, arguments)
    }

    /// A call of a function of a module loaded from a file, which is a static method of it.
    ///
    /// What it takes and gives back is read off the use rather than off the other module's
    /// tree, which this module never holds. Inference has already met the two, so the use
    /// carries the very signature the other module wrote the method with.
    fn in_another_module(
        &mut self,
        reached: &Through<'_>,
        arguments: &[&Expr],
    ) -> Option<Descriptor> {
        let (named, signature) = self.reaches(reached);
        for (argument, wanted) in arguments.iter().zip(&signature.parameters) {
            self.handed(argument, wanted.clone());
        }
        self.emit(Instruction::InvokeStatic(MethodRef {
            class: ClassName::new(&reached.module.text),
            name: named,
            descriptor: signature.descriptor(),
        }));
        signature.result
    }

    /// The method this use reaches: what the other module calls it, and what it is written with.
    ///
    /// A function that declares no type parameter is written once, under its own name, and the
    /// use carries the very signature the other module wrote it with. A generic is written once
    /// per set of types it is used at, which `docs/specs/codegen.md` states, so a use of one
    /// reaches a method named for the set this use settled and written with the type that set
    /// gives the declaration, which is not the type the use has.
    ///
    /// A use inside a generic settles on that generic's own type parameters, and the body being
    /// written is one set of those, so the set asked for is what this set gives them.
    fn reaches(&mut self, reached: &Through<'_>) -> (String, Signature) {
        let Some(generic) = self.lowering.typed.generic_reached(reached.name.span) else {
            let signature = self.lowering.reached_through(reached.used);
            return (reached.name.text.clone(), signature);
        };
        let module = &reached.module.text;
        let settled: Vec<Type> = generic
            .settled()
            .iter()
            .map(|at| self.at().substituted(at))
            .collect();
        self.owes_each_instance(generic.constrained(), &settled);
        let asked = Asking {
            settled: settled
                .iter()
                .map(|at| self.lowering.shapes.as_written_by(module, at))
                .collect(),
            constrained: generic.constrained().to_vec(),
        };
        let named = self.lowering.asking(module, &reached.name.text, asked);
        let written_as = self.at().substituted(generic.written_as());
        (named, self.lowering.written_as(&written_as))
    }

    /// Owes the instance method the other module's generic calls at each type this use settled.
    ///
    /// The method is written there and the instance is written here, so this is the module that
    /// can write it: the other one is handed a type and names the call from it, and only this one
    /// knows what the instance's own type parameters settled on. `docs/specs/codegen.md` states
    /// the rule, and an instance over a type written by name alone is written whether anything
    /// asks for it or not, so asking for it again costs nothing.
    fn owes_each_instance(&self, constrained: &[Vec<String>], settled: &[Type]) {
        for (traits, at) in constrained.iter().zip(settled) {
            for of in traits {
                let method = prelude::method_of(of)
                    .expect("a trait two modules both name the prelude declares");
                self.lowering.answering(method, at);
            }
        }
    }
}

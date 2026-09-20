//! What a whole-number literal is written as, which is the number or a call that takes it.
//!
//! `docs/specs/literals.md` states it: a type the compiler supplies the instance for takes the
//! number itself, which is what a literal has always been, and a type whose instance a module
//! wrote takes it through that instance's `from_literal`, so a literal costs one call and no more.

use lumen_ast::{Expr, Span};
use lumen_resolver::prelude;

use crate::code::{Instruction, MethodRef};
use crate::descriptor::Descriptor;
use crate::lower::body::Builder;

impl Builder<'_> {
    /// A whole number, at whichever type inference settled it on.
    pub(crate) fn whole_number(&mut self, value: i64, written: Span) -> Descriptor {
        self.emit(Instruction::Long(value));
        let Some(declared) = self.instance_written(prelude::FROM_LITERAL, written) else {
            return Descriptor::Long;
        };
        let reached = self.reaching(declared, written);
        self.emit(Instruction::InvokeStatic(MethodRef {
            class: self.lowering.shapes.module().clone(),
            name: reached.named,
            descriptor: reached.signature.descriptor(),
        }));
        reached
            .signature
            .result
            .expect("`from_literal` gives back the type the literal is written at")
    }

    /// `from_literal(value)` at a type the compiler supplies the instance for, which is `Int`.
    ///
    /// The supplied instance turns a whole number into the whole number it already is, so the
    /// call is the value and nothing more, exactly as the literal `5` is the number `5`.
    pub(crate) fn supplied_from_literal(&mut self, arguments: &[&Expr]) -> Descriptor {
        let [value] = arguments else {
            unreachable!("inference gave `from_literal` the one argument it takes")
        };
        self.expr(value)
            .expect("`from_literal` takes an `Int`, which is a value")
    }
}

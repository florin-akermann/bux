//! The prelude's own functions, written out where they are called.
//!
//! The prelude is not Lumen source yet, so there is no class to call one on. `or` is the one
//! that lowers, and the two branches it amounts to are written here. `todo` is the other, and
//! nothing lowers a hole: `docs/specs/holes.md` has `lumen build` refuse every one of them
//! before a single class file is written.

use lumen_ast::{Expr, Name, Span};

use crate::code::{Comparison, FieldRef, Instruction, Label};
use crate::descriptor::Descriptor;
use crate::lower::body::{Builder, Slot};
use crate::lower::shape::{SOME, TAG};

/// What the prelude calls the one function version 0.1 lowers.
const OR: &str = "or";

impl Builder<'_> {
    /// A call of a function the prelude supplies, which version 0.1 writes out where it is used.
    ///
    /// `or` is the only one that is lowered. There is no class to call it on, because the
    /// prelude is not yet Lumen source, so the two branches it amounts to are written here
    /// instead. `todo` is the other, and nothing lowers a hole: `docs/specs/holes.md` has
    /// `lumen build` refuse every one of them before a single class file is written.
    pub(crate) fn supplied(
        &mut self,
        name: &Name,
        arguments: &[&Expr],
        written: Span,
    ) -> Option<Descriptor> {
        assert!(
            name.text == OR,
            "`or` is the one prelude function that lowers; `Whole` keeps a hole from reaching here"
        );
        let wanted = self.carried(written);
        let [maybe, fallback] = arguments else {
            unreachable!("inference gave `or` the two arguments it takes")
        };
        let held = self.set_aside(maybe);
        let otherwise_held = self.set_aside_as(fallback, wanted.clone());
        let otherwise = self.label();
        let end = self.label();
        self.carrying(SOME, &held, otherwise);
        self.read_carried(SOME, &held, wanted.clone());
        self.emit(Instruction::Jump(end));
        self.emit(Instruction::Label(otherwise));
        self.reload(otherwise_held.as_ref());
        self.emit(Instruction::Label(end));
        wanted
    }

    /// Puts the `Option` in a local, because the branch that reads it must load it again.
    fn set_aside(&mut self, maybe: &Expr) -> Slot {
        let Some(of) = self.carried(maybe.span) else {
            unreachable!("`or` is handed an `Option`, which a reference always carries")
        };
        let left = self.expr(maybe);
        self.adapt(left, Some(of.clone()));
        let at = self.temporary(&of);
        self.emit(Instruction::Store {
            slot: at,
            of: of.clone(),
        });
        Slot { at, of }
    }

    /// Puts the fallback in a local, as `wanted` wants it, before either branch is taken.
    ///
    /// A call evaluates its arguments, and `or` is a call: the prelude becomes Lumen source
    /// later, and a program must not change its behaviour when it does. Nothing is left in a
    /// local when the fallback is carried by nothing, because there is nothing to leave.
    fn set_aside_as(&mut self, fallback: &Expr, wanted: Option<Descriptor>) -> Option<Slot> {
        self.handed(fallback, wanted.clone());
        let of = wanted?;
        let at = self.temporary(&of);
        self.emit(Instruction::Store {
            slot: at,
            of: of.clone(),
        });
        Some(Slot { at, of })
    }

    /// Reads back what was put aside, which is nothing when nothing was.
    fn reload(&mut self, held: Option<&Slot>) {
        if let Some(slot) = held {
            self.emit(Instruction::Load {
                slot: slot.at,
                of: slot.of.clone(),
            });
        }
    }

    /// Jumps to `otherwise` unless the value in `held` is the variant called `variant`.
    fn carrying(&mut self, variant: &str, held: &Slot, otherwise: Label) {
        let shape = self.lowering.shapes.built(variant).clone();
        let Some(tag) = shape.tag else {
            unreachable!("every variant of a prelude type carries a tag")
        };
        self.emit(Instruction::Load {
            slot: held.at,
            of: held.of.clone(),
        });
        self.emit(Instruction::Cast(shape.base.clone()));
        self.emit(Instruction::GetField(FieldRef {
            class: shape.base,
            name: TAG.to_owned(),
            of: Descriptor::Integer,
        }));
        self.emit(Instruction::Integer(tag));
        self.emit(Instruction::CompareIntegers(Comparison::Equal));
        self.emit(Instruction::JumpIfFalse(otherwise));
    }

    /// Reads the one value the variant carries, as whatever the call's own type wants it.
    fn read_carried(&mut self, variant: &str, held: &Slot, wanted: Option<Descriptor>) {
        let shape = self.lowering.shapes.built(variant).clone();
        let Some(carried) = shape.carries.first() else {
            unreachable!("`Some` carries the one value `or` gives back")
        };
        let Some(of) = carried.of.clone() else {
            unreachable!("`Some` carries its value as a reference, whatever the value is")
        };
        self.emit(Instruction::Load {
            slot: held.at,
            of: held.of.clone(),
        });
        self.emit(Instruction::Cast(shape.class.clone()));
        self.emit(Instruction::GetField(FieldRef {
            class: shape.class,
            name: carried.name.clone(),
            of: of.clone(),
        }));
        self.adapt(Some(of), wanted);
    }
}

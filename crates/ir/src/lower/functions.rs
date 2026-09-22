//! The prelude's own functions, written out where they are called.
//!
//! `or` and `ok_or` are the ones that lower, and the two branches each amounts to are written
//! here rather than called on the prelude's class. `todo` is the other, and nothing lowers a
//! hole: `docs/specs/holes.md` has `lumen build` refuse every one of them before a class file is
//! written. The prelude's own source also calls `at` by its bare name, and that is the list read
//! the compiler holds.

use lumen_ast::{Expr, Name, Span};

use crate::code::{Comparison, FieldRef, Instruction, Label};
use crate::descriptor::Descriptor;
use crate::lower::body::{Builder, Slot};
use crate::lower::shape::{OK, SOME, TAG};

/// What the prelude calls reading an `Option` with a fallback.
const OR: &str = "or";

/// What the prelude calls reading a `Result` with a fallback.
const OK_OR: &str = "ok_or";

impl Builder<'_> {
    /// A call of a function the prelude supplies, which version 0.1 writes out where it is used.
    ///
    /// `or` and `ok_or` are the ones that are lowered, and they are one body over which variant
    /// carries the value: `Some` for the first and `Ok` for the second. Each is written where it
    /// is called, and the prelude's class writes only its instances. `todo` is the other, and
    /// nothing lowers a hole: `docs/specs/holes.md` has `lumen build` refuse every one of them
    /// before a single class file is written. The bare `at` of the prelude's own source is the
    /// list read the compiler holds.
    pub(crate) fn supplied(
        &mut self,
        name: &Name,
        arguments: &[&Expr],
        written: Span,
    ) -> Option<Descriptor> {
        if let Some(held) = self.held_named(&name.text, arguments) {
            return Some(held);
        }
        let carrying = match name.text.as_str() {
            OR => SOME,
            OK_OR => OK,
            named => unreachable!(
                "`{named}` is no prelude function that lowers; `Whole` keeps a hole from here"
            ),
        };
        let wanted = self.carried(written);
        let [answer, fallback] = arguments else {
            unreachable!("inference gave `{}` the two arguments it takes", name.text)
        };
        let held = self.set_aside(answer);
        let otherwise_held = self.set_aside_as(fallback, wanted.clone());
        let otherwise = self.label();
        let end = self.label();
        self.carrying(carrying, &held, otherwise);
        self.read_carried(carrying, &held, wanted.clone());
        self.emit(Instruction::Jump(end));
        self.emit(Instruction::Label(otherwise));
        self.reload(otherwise_held.as_ref());
        self.emit(Instruction::Label(end));
        wanted
    }

    /// Puts the answer in a local, because the branch that reads it must load it again.
    fn set_aside(&mut self, answer: &Expr) -> Slot {
        let Some(of) = self.carried(answer.span) else {
            unreachable!("an answer read with a fallback is one a reference always carries")
        };
        let left = self.expr(answer);
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
            unreachable!("the variant read with a fallback carries the one value it gives back")
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

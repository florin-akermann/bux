//! What a `match` does: each arm in the order it is written, and what it binds when it matches.
//!
//! An arm tests what it must and falls to the next arm when the test fails. Exhaustiveness has
//! already proved that some arm answers for every value, so falling past the last one is a thing
//! that cannot happen, and the method says so by throwing rather than by carrying on.

use lumen_ast::{MatchExpr, Name, Pattern, PatternKind, Span};
use lumen_resolver::DefinitionKind;

use crate::code::{Comparison, FieldRef, Instruction, Label, MethodRef};
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
use crate::lower::body::{Builder, Slot};
use crate::lower::shape::{CONSTRUCTOR, Carried, Shape, TAG};
use crate::lower::supplied::compared;

/// What a pattern is matched against: the value, and where to go when it does not match.
struct Against {
    slot: Slot,
    next: Label,
}

impl Builder<'_> {
    /// Lowers a `match`, leaving the value of whichever arm answered for what it was given.
    pub(crate) fn match_expr(&mut self, matched: &MatchExpr, at: Span) -> Option<Descriptor> {
        let wanted = self.carried(at);
        let held = self.matched(matched);
        let end = self.label();
        for arm in &matched.arms {
            let next = self.label();
            self.tried(&arm.pattern, held.as_ref(), next);
            let left = self.expr(&arm.body);
            self.adapt(left, wanted.clone());
            self.emit(Instruction::Jump(end));
            self.emit(Instruction::Label(next));
        }
        self.unmatched();
        self.emit(Instruction::Label(end));
        wanted
    }

    /// Puts what is being matched in a local, so that every arm reads the same one value.
    fn matched(&mut self, matched: &MatchExpr) -> Option<Slot> {
        let of = self.carried(matched.scrutinee.span);
        let left = self.expr(&matched.scrutinee);
        self.adapt(left, of.clone());
        let of = of?;
        let at = self.temporary(&of);
        self.emit(Instruction::Store {
            slot: at,
            of: of.clone(),
        });
        Some(Slot { at, of })
    }

    /// Nothing reaches this, because every value has an arm; the method says so rather than run on.
    fn unmatched(&mut self) {
        let unreachable = ClassName::new("java/lang/AssertionError");
        self.emit(Instruction::New(unreachable.clone()));
        self.emit(Instruction::Copy);
        self.emit(Instruction::Construct(MethodRef {
            class: unreachable,
            name: CONSTRUCTOR.to_owned(),
            descriptor: MethodDescriptor::new(Vec::new(), None),
        }));
        self.emit(Instruction::Throw);
    }

    /// Tests `pattern` against the value, jumping to `next` as soon as it does not match.
    fn tried(&mut self, pattern: &Pattern, held: Option<&Slot>, next: Label) {
        let Some(slot) = held else {
            return;
        };
        let against = Against {
            slot: slot.clone(),
            next,
        };
        match &pattern.kind {
            PatternKind::Name(name) => self.tried_name(name, &against),
            PatternKind::Tuple { name, elements } => self.tried_tuple(name, elements, &against),
            PatternKind::Record { name, fields } => self.tried_record(name, fields, &against),
            PatternKind::Integer(value) => {
                self.equal_to(Instruction::Long(*value), &Descriptor::Long, &against);
            }
            PatternKind::Bool(value) => {
                self.equal_to(Instruction::Boolean(*value), &Descriptor::Boolean, &against);
            }
            PatternKind::String(value) => self.equal_to(
                Instruction::Text(value.clone()),
                &Descriptor::reference("java/lang/String"),
                &against,
            ),
        }
    }

    /// A bare name is a variant that carries nothing, or a binding that matches anything.
    fn tried_name(&mut self, name: &Name, against: &Against) {
        if self.definition(name).kind == DefinitionKind::Constructor {
            self.tagged(name, against);
            return;
        }
        self.declare(name);
        self.emit(Instruction::Load {
            slot: against.slot.at,
            of: against.slot.of.clone(),
        });
        self.stored(name, Some(against.slot.of.clone()));
    }

    /// `Failed(reason)`: the value is a `Failed`, and what it carries matches in order.
    fn tried_tuple(&mut self, name: &Name, elements: &[Pattern], against: &Against) {
        let shape = self.tagged(name, against);
        for (element, carried) in elements.iter().zip(&shape.carries) {
            let Some(of) = carried.of.clone() else {
                continue;
            };
            self.read(&shape, carried, against);
            let at = self.temporary(&of);
            self.emit(Instruction::Store {
                slot: at,
                of: of.clone(),
            });
            self.tried(element, Some(&Slot { at, of }), against.next);
        }
    }

    /// `Authorized { authorization_id }`: the value is an `Authorized`, and each field binds.
    fn tried_record(&mut self, name: &Name, fields: &[Name], against: &Against) {
        let shape = self.tagged(name, against);
        for field in fields {
            let Some(carried) = shape
                .carries
                .iter()
                .find(|carried| carried.name == field.text)
            else {
                unreachable!("inference gave every field of a pattern a variant that declares it")
            };
            let held = carried.of.clone();
            self.declare(field);
            self.read(&shape, carried, against);
            self.stored(field, held);
        }
    }

    /// The value is one `name` built, which its tag says; a type with one of them needs no test.
    fn tagged(&mut self, name: &Name, against: &Against) -> Shape {
        let shape = self.lowering.shapes.built(&name.text).clone();
        let Some(tag) = shape.tag else {
            return shape;
        };
        self.emit(Instruction::Load {
            slot: against.slot.at,
            of: against.slot.of.clone(),
        });
        self.emit(Instruction::Cast(shape.base.clone()));
        self.emit(Instruction::GetField(FieldRef {
            class: shape.base.clone(),
            name: TAG.to_owned(),
            of: Descriptor::Integer,
        }));
        self.emit(Instruction::Integer(tag));
        self.emit(Instruction::CompareIntegers(Comparison::Equal));
        self.emit(Instruction::JumpIfFalse(against.next));
        shape
    }

    /// Reads one of the values the variant carries off the value being matched.
    fn read(&mut self, shape: &Shape, carried: &Carried, against: &Against) {
        let Some(of) = carried.of.clone() else {
            return;
        };
        self.emit(Instruction::Load {
            slot: against.slot.at,
            of: against.slot.of.clone(),
        });
        self.emit(Instruction::Cast(shape.class.clone()));
        self.emit(Instruction::GetField(FieldRef {
            class: shape.class.clone(),
            name: carried.name.clone(),
            of,
        }));
    }

    /// The value is the literal the pattern writes, which is what `==` would have asked.
    fn equal_to(&mut self, literal: Instruction, of: &Descriptor, against: &Against) {
        self.emit(Instruction::Load {
            slot: against.slot.at,
            of: against.slot.of.clone(),
        });
        self.adapt(Some(against.slot.of.clone()), Some(of.clone()));
        self.emit(literal);
        for instruction in compared(Some(of), Comparison::Equal) {
            self.emit(instruction);
        }
        self.emit(Instruction::JumpIfFalse(against.next));
    }
}

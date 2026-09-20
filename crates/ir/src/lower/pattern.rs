//! What a `match` does: each arm in the order it is written, and what it binds when it matches.
//!
//! An arm tests what it must and falls to the next arm when the test fails. Exhaustiveness has
//! already proved that some arm answers for every value, so falling past the last one is a thing
//! that cannot happen, and the method says so by throwing rather than by carrying on.

use lumen_ast::{MatchExpr, Name, Path, Pattern, PatternKind, Span};
use lumen_resolver::DefinitionKind;

use lumen_resolver::prelude;

use crate::code::{Comparison, FieldRef, Instruction, Label, MethodRef};
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
use crate::lower::body::{Builder, Slot};
use crate::lower::shape::{CONSTRUCTOR, Carried, Shape, TAG};
use crate::lower::standard::compared;

/// What a pattern is matched against: the value, and where to go when it does not match.
struct Against {
    slot: Slot,
    next: Label,
}

/// A literal a pattern writes: the instruction that pushes it, and the type that instruction is.
struct Literal {
    pushed: Instruction,
    of: Descriptor,
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
        let written = pattern.span;
        match &pattern.kind {
            PatternKind::Name(path) => self.tried_name(path, &against),
            PatternKind::Tuple { path, elements } => self.tried_tuple(path, elements, &against),
            PatternKind::Record { path, fields } => self.tried_record(path, fields, &against),
            PatternKind::Or(alternatives) => self.tried_any_of(alternatives, &against),
            PatternKind::Integer(value) => self.tried_number(*value, written, &against),
            PatternKind::Bool(value) => {
                let written_as = Literal {
                    pushed: Instruction::Boolean(*value),
                    of: Descriptor::Boolean,
                };
                self.equal_to(written_as, written, &against);
            }
            PatternKind::String(value) => {
                let written_as = Literal {
                    pushed: Instruction::Text(value.clone()),
                    of: Descriptor::reference("java/lang/String"),
                };
                self.equal_to(written_as, written, &against);
            }
            PatternKind::Wildcard => {}
        }
    }

    /// `Pending | Running`: each alternative in turn, and the first that matches answers.
    ///
    /// No alternative binds, which `docs/specs/patterns.md` states, so the arm below reads the
    /// same locals whichever one of them the value took.
    fn tried_any_of(&mut self, alternatives: &[Pattern], against: &Against) {
        let matched = self.label();
        let Some((last, rest)) = alternatives.split_last() else {
            return;
        };
        for alternative in rest {
            let next = self.label();
            self.tried(alternative, Some(&against.slot), next);
            self.emit(Instruction::Jump(matched));
            self.emit(Instruction::Label(next));
        }
        self.tried(last, Some(&against.slot), against.next);
        self.emit(Instruction::Label(matched));
    }

    /// A whole number, which is the number at the type it is matched against.
    ///
    /// `docs/specs/patterns.md` holds it to `docs/specs/literals.md`, so a type whose instance a
    /// module wrote takes the number through that instance's `from_literal` here too.
    fn tried_number(&mut self, value: i64, written: Span, against: &Against) {
        let of = self.carried(written);
        self.emit(Instruction::Load {
            slot: against.slot.at,
            of: against.slot.of.clone(),
        });
        self.adapt(Some(against.slot.of.clone()), of);
        let pushed = self.whole_number(value, written);
        self.same_as(&pushed, written, against);
    }

    /// A bare name is a variant that carries nothing, or a binding that matches anything.
    ///
    /// One reached through a module is always the variant: nothing binds a name of another.
    fn tried_name(&mut self, path: &Path, against: &Against) {
        if path.module.is_some() || self.definition(&path.name).kind == DefinitionKind::Constructor
        {
            self.tagged(path, against);
            return;
        }
        let name = &path.name;
        self.declare(name);
        self.emit(Instruction::Load {
            slot: against.slot.at,
            of: against.slot.of.clone(),
        });
        self.stored(name, Some(against.slot.of.clone()));
    }

    /// `Failed(reason)`: the value is a `Failed`, and what it carries matches in order.
    fn tried_tuple(&mut self, path: &Path, elements: &[Pattern], against: &Against) {
        let shape = self.tagged(path, against);
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
    fn tried_record(&mut self, path: &Path, fields: &[Name], against: &Against) {
        let shape = self.tagged(path, against);
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

    /// The value is one `path` built, which its tag says; a type with one of them needs no test.
    fn tagged(&mut self, path: &Path, against: &Against) -> Shape {
        let shape = self.lowering.shapes.built(&path.to_string()).clone();
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
    fn equal_to(&mut self, literal: Literal, written: Span, against: &Against) {
        self.emit(Instruction::Load {
            slot: against.slot.at,
            of: against.slot.of.clone(),
        });
        self.adapt(Some(against.slot.of.clone()), Some(literal.of.clone()));
        self.emit(literal.pushed);
        self.same_as(&literal.of, written, against);
    }

    /// Whether the two values above it are the same, which is the `Eq` of the type they are.
    ///
    /// A pattern asks what `==` asks, so it reaches the instance `==` reaches: one a module
    /// wrote is a call of its method, and one over a type the JVM holds is the instruction it is.
    fn same_as(&mut self, held: &Descriptor, written: Span, against: &Against) {
        match self.instance_written(prelude::IS_EQUAL, written) {
            None => {
                for instruction in compared(Some(held), Comparison::Equal) {
                    self.emit(instruction);
                }
            }
            Some(declared) => {
                let reached = self.reaching(declared, written);
                self.emit(Instruction::InvokeStatic(MethodRef {
                    class: self.lowering.shapes.module().clone(),
                    name: reached.named,
                    descriptor: reached.signature.descriptor(),
                }));
            }
        }
        self.emit(Instruction::JumpIfFalse(against.next));
    }
}

//! What an operator is written as, which is a call of the method its trait declares.
//!
//! `docs/specs/operators.md` names the trait each operator is and states this: a type whose
//! instance a module wrote gets an `invokestatic` of that instance's method, and one of the
//! types the JVM holds gets the instruction the operator always was, written out in place.

use lumen_ast::{BinaryOperator, Expr, Name, UnaryOperator};
use lumen_resolver::prelude;

use crate::code::{Arithmetic, Comparison, Instruction};
use crate::descriptor::Descriptor;
use crate::lower::Instance;
use crate::lower::body::{Builder, Slot};
use crate::lower::standard::{compared, hashed_as, shown_as};

impl Builder<'_> {
    /// An operator, which is the call of the one method its trait declares.
    ///
    /// `&&` and `||` never reach here: each decides whether to run the other side, which no call
    /// can, so `docs/specs/operators.md` keeps them `Bool`'s alone and the caller writes them.
    pub(crate) fn operated(
        &mut self,
        operator: BinaryOperator,
        left: &Expr,
        right: &Expr,
    ) -> Option<Descriptor> {
        let asked = asked_by(operator).expect("`&&` and `||` are written before this is reached");
        let Some(instance) = self.instance_written(method_of(asked.of), left.span) else {
            return self.written_out_operator(operator, left, right);
        };
        Some(self.through_the_operator(&asked, &instance, [left, right]))
    }

    /// A direct call of one of the prelude's trait methods, at a type the prelude has an
    /// instance for.
    ///
    /// `add(one, other)` is the call `one + other` already is, so the two are written the same.
    /// `from_literal(5)` is the whole number `5` already is, which `lower/literal.rs` writes.
    pub(crate) fn written_out_instance(&mut self, name: &Name, arguments: &[&Expr]) -> Descriptor {
        let of = prelude::trait_of(&name.text)
            .expect("the prelude declares every method that reaches here");
        if of == prelude::INTEGER_LITERAL {
            return self.written_out_from_literal(arguments);
        }
        if let Some(reads) = reading_one(of) {
            let [value] = arguments else {
                unreachable!("inference gave `{}` the one argument it takes", name.text)
            };
            return self.read_one(reads, value);
        }
        if of == prelude::NEG {
            let [value] = arguments else {
                unreachable!("inference gave `negate` the one argument it takes")
            };
            return self.negated(value);
        }
        let [left, right] = arguments else {
            unreachable!("inference gave `{}` the two arguments it takes", name.text)
        };
        self.written_out_operator(written_as(of), left, right)
            .expect("every supplied operator leaves a value")
    }

    /// `hashed(value)` and `shown(value)` at a type the prelude has an instance for, each of
    /// which reads the one value it is handed.
    fn read_one(&mut self, reads: ReadsOne, value: &Expr) -> Descriptor {
        let held = self
            .value(value)
            .expect("a type with `Hash` or `Show` is carried by something");
        let (written, gives) = match reads {
            ReadsOne::Hashed => (hashed_as(&held), Descriptor::Long),
            ReadsOne::Shown => (shown_as(&held), Descriptor::reference("java/lang/String")),
        };
        for instruction in written {
            self.emit(instruction);
        }
        gives
    }

    /// `total += value` is `Add`, so it asks the trait `total + value` asks.
    ///
    /// What the name holds is loaded first, because it is the first of the two the method takes,
    /// and one of the types the JVM holds adds as that operator always added.
    pub(crate) fn added_to(&mut self, slot: &Slot, value: &Expr) -> Descriptor {
        self.emit(Instruction::Load {
            slot: slot.at,
            of: slot.of.clone(),
        });
        let Some(instance) = self.instance_written(method_of(prelude::ADD), value.span) else {
            let left = self.expr(value);
            self.adapt(left, Some(slot.of.clone()));
            self.emit(written_out_add(&slot.of));
            return slot.of.clone();
        };
        self.handed(value, instance.signature().parameters[1].clone());
        self.emit(instance.called());
        instance
            .signature()
            .result
            .clone()
            .expect("`add` gives back the type it was given, which a local holds")
    }

    /// Prefix `-` is `Neg`, so a type whose instance a module wrote negates by that instance.
    pub(crate) fn negated(&mut self, operand: &Expr) -> Descriptor {
        let Some(instance) = self.instance_written(method_of(prelude::NEG), operand.span) else {
            let held = self.expr(operand);
            self.adapt(held, Some(Descriptor::Long));
            self.emit(Instruction::Arithmetic(Arithmetic::Negate));
            return Descriptor::Long;
        };
        self.calling(&instance, &[operand])
            .expect("`negate` gives back the type it was given, which is carried by something")
    }

    /// An operator over a type whose instance a module wrote, which is a call of that method.
    fn through_the_operator(
        &mut self,
        asked: &Asked,
        instance: &Instance,
        operands: [&Expr; 2],
    ) -> Descriptor {
        let given = if asked.reversed {
            self.the_other_way_round(instance, operands)
        } else {
            self.calling(instance, &operands)
        };
        if asked.flipped {
            self.emit(Instruction::Not);
        }
        given.expect("every operator's method gives back a value")
    }

    /// `one > other` is `is_less(other, one)`, which `docs/specs/operators.md` states.
    ///
    /// Both sides are run in the order they are written and kept in locals, because a side may
    /// be a call and the method takes the two the other way round.
    fn the_other_way_round(
        &mut self,
        instance: &Instance,
        operands: [&Expr; 2],
    ) -> Option<Descriptor> {
        let mut kept = Vec::new();
        for (operand, wanted) in operands.iter().zip(&instance.signature().parameters) {
            let Some(of) = wanted.clone() else {
                self.value(operand);
                continue;
            };
            kept.push((self.put_aside_as(operand, &of), of));
        }
        kept.reverse();
        for (slot, of) in kept {
            self.emit(Instruction::Load { slot, of });
        }
        self.emit(instance.called());
        instance.signature().result.clone()
    }

    /// What the prelude's instance amounts to, written out where the operator is called.
    ///
    /// `Int` has every operator and `String` has `+`, which is the whole of what the prelude
    /// writes an operator for, and what each writes is what that operator always wrote.
    pub(crate) fn written_out_operator(
        &mut self,
        operator: BinaryOperator,
        left: &Expr,
        right: &Expr,
    ) -> Option<Descriptor> {
        if matches!(operator, BinaryOperator::Divide | BinaryOperator::Remainder) {
            return Some(self.divided(counted(operator), left, right));
        }
        let held = self.value(left);
        let other = self.value(right);
        self.adapt(other, held.clone());
        match operator {
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Less
            | BinaryOperator::LessOrEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterOrEqual => Some(self.same(operator, held.as_ref())),
            BinaryOperator::Add => self.added(held),
            BinaryOperator::Subtract | BinaryOperator::Multiply => {
                self.emit(Instruction::Arithmetic(counted(operator)));
                Some(Descriptor::Long)
            }
            BinaryOperator::Or
            | BinaryOperator::And
            | BinaryOperator::Divide
            | BinaryOperator::Remainder => {
                unreachable!("`&&`, `||`, `/`, and `%` are each written before this is reached")
            }
        }
    }

    /// What the two values above it on the stack compare to, for the prelude's `Eq` or `Ord`.
    fn same(&mut self, operator: BinaryOperator, held: Option<&Descriptor>) -> Descriptor {
        for instruction in compared(held, how(operator)) {
            self.emit(instruction);
        }
        Descriptor::Boolean
    }
}

/// The one standard trait whose method reads one value rather than comparing two.
#[derive(Clone, Copy)]
enum ReadsOne {
    Hashed,
    Shown,
}

/// Which of the two `of` is, or nothing where its method takes two values.
fn reading_one(of: &str) -> Option<ReadsOne> {
    match of {
        prelude::HASH => Some(ReadsOne::Hashed),
        prelude::SHOW => Some(ReadsOne::Shown),
        _ => None,
    }
}

/// How an operator's call is read: the trait it is, and which way round its answer is.
pub(crate) struct Asked {
    /// The trait the operator is, which `docs/specs/operators.md` names.
    of: &'static str,
    /// Whether the method is given the right side first, which `>` and `<=` are.
    reversed: bool,
    /// Whether the answer is the other one, which `!=`, `<=`, and `>=` take.
    flipped: bool,
}

/// The trait each operator is, and how it reads, or nothing where it is `Bool`'s alone.
const fn asked_by(operator: BinaryOperator) -> Option<Asked> {
    let asked = match operator {
        BinaryOperator::Or | BinaryOperator::And => return None,
        BinaryOperator::Equal => reading(prelude::EQ, false, false),
        BinaryOperator::NotEqual => reading(prelude::EQ, false, true),
        BinaryOperator::Less => reading(prelude::ORD, false, false),
        BinaryOperator::Greater => reading(prelude::ORD, true, false),
        BinaryOperator::LessOrEqual => reading(prelude::ORD, true, true),
        BinaryOperator::GreaterOrEqual => reading(prelude::ORD, false, true),
        BinaryOperator::Add => reading(prelude::ADD, false, false),
        BinaryOperator::Subtract => reading(prelude::SUB, false, false),
        BinaryOperator::Multiply => reading(prelude::MUL, false, false),
        BinaryOperator::Divide => reading(prelude::DIV, false, false),
        BinaryOperator::Remainder => reading(prelude::REM, false, false),
    };
    Some(asked)
}

const fn reading(of: &'static str, reversed: bool, flipped: bool) -> Asked {
    Asked {
        of,
        reversed,
        flipped,
    }
}

/// The one method the trait `of` declares, which is the call an operator is.
fn method_of(of: &str) -> &'static str {
    prelude::method_of(of).expect("every operator's trait declares the one method it is")
}

/// The operator the trait `of` is, which is what the prelude's instance of it writes out.
fn written_as(of: &str) -> BinaryOperator {
    match of {
        prelude::EQ => BinaryOperator::Equal,
        prelude::ORD => BinaryOperator::Less,
        prelude::ADD => BinaryOperator::Add,
        prelude::SUB => BinaryOperator::Subtract,
        prelude::MUL => BinaryOperator::Multiply,
        prelude::DIV => BinaryOperator::Divide,
        prelude::REM => BinaryOperator::Remainder,
        _ => unreachable!("prefix `-` and a literal are both written before this is reached"),
    }
}

/// Which comparison the prelude's `Eq` or `Ord` asks for.
const fn how(operator: BinaryOperator) -> Comparison {
    match operator {
        BinaryOperator::NotEqual => Comparison::NotEqual,
        BinaryOperator::Less => Comparison::Less,
        BinaryOperator::LessOrEqual => Comparison::LessOrEqual,
        BinaryOperator::Greater => Comparison::Greater,
        BinaryOperator::GreaterOrEqual => Comparison::GreaterOrEqual,
        BinaryOperator::Equal
        | BinaryOperator::Or
        | BinaryOperator::And
        | BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide
        | BinaryOperator::Remainder => Comparison::Equal,
    }
}

/// What an operator does to two whole numbers.
const fn counted(operator: BinaryOperator) -> Arithmetic {
    match operator {
        BinaryOperator::Subtract => Arithmetic::Subtract,
        BinaryOperator::Multiply => Arithmetic::Multiply,
        BinaryOperator::Divide => Arithmetic::Divide,
        BinaryOperator::Remainder => Arithmetic::Remainder,
        BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::Less
        | BinaryOperator::LessOrEqual
        | BinaryOperator::Greater
        | BinaryOperator::GreaterOrEqual
        | BinaryOperator::Or
        | BinaryOperator::And
        | BinaryOperator::Add => Arithmetic::Add,
    }
}

/// `!` is `Bool`'s alone, which is why it is the one unary operator with no trait.
pub(crate) const fn is_negation(operator: UnaryOperator) -> bool {
    matches!(operator, UnaryOperator::Negate)
}

/// What the prelude's `Add` writes, which joins two strings and adds two whole numbers.
fn written_out_add(of: &Descriptor) -> Instruction {
    match of {
        Descriptor::Reference(_) | Descriptor::Array(_) => Instruction::Concat,
        Descriptor::Long | Descriptor::Boolean | Descriptor::Integer | Descriptor::Character => {
            Instruction::Arithmetic(Arithmetic::Add)
        }
    }
}

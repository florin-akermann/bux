//! What the prelude's instances of the four standard traits over `List` amount to.
//!
//! `library/prelude.lm` writes `Eq`, `Ord`, `Hash`, and `Show` for `List<T>` in plain Lumen, each
//! constrained on what `T` answers, and `docs/specs/traits.md` states them. A list is read by a
//! loop rather than by one instruction, so each of these is a method rather than instructions
//! written out where they stand: the class of the module that uses it holds it, which
//! `docs/specs/codegen.md` states, and every element is handed to the instance of its own type.

use lumen_resolver::prelude;

use crate::code::{Arithmetic, Body, Comparison, Instruction, Label};
use crate::descriptor::Descriptor;
use crate::lower::body::{LIST, reaching, read_back_as};
use crate::lower::shape::object;
use crate::lower::standard::{compared, hashed_as, shown_as};
use crate::lower::{Lowering, OverAList, Signature};

/// The local the first list one of these methods takes arrives in.
const ONE: u16 = 0;

/// The local the second arrives in, which `Hash` and `Show` never have.
const OTHER: u16 = 1;

/// What a polynomial hash multiplies by at each step, which is the textbook multiplier.
const MULTIPLIER: i64 = 31;

/// The text a shown list opens with, which is what a written list opens with.
const OPENING: &str = "[";

/// The text a shown list writes between two elements.
const BETWEEN: &str = ", ";

/// The text a shown list closes with.
const CLOSING: &str = "]";

/// The body of the method the instance `over` names writes, at the type its list holds.
pub(crate) fn body(lowering: &Lowering<'_>, over: &OverAList, signature: &Signature) -> Body {
    let mut writing = Writing {
        lowering,
        over,
        held: carried_by(lowering, over),
        instructions: Vec::new(),
        next_label: 0,
        next_slot: taken_by(signature),
        locals: 0,
    };
    match over.of.as_str() {
        prelude::EQ => writing.is_equal(),
        prelude::ORD => writing.is_less(),
        prelude::HASH => writing.hashed(),
        prelude::SHOW => writing.shown(),
        of => unreachable!("`{of}` is no trait the prelude writes an instance over `List` of"),
    }
    writing.emit(Instruction::Return(signature.result.clone()));
    Body {
        instructions: writing.instructions,
        locals: writing.locals,
        guards: Vec::new(),
    }
}

/// One such body part-way through being written.
struct Writing<'a> {
    lowering: &'a Lowering<'a>,
    over: &'a OverAList,
    /// What the JVM carries one element as, once it is read back out of the list.
    held: Descriptor,
    instructions: Vec<Instruction>,
    next_label: u32,
    next_slot: u16,
    locals: u16,
}

impl Writing<'_> {
    /// `Eq`: the two lists are the same length, and every element equals the one beside it.
    fn is_equal(&mut self) {
        let differs = self.label();
        self.size_of(ONE);
        self.size_of(OTHER);
        self.emit(Instruction::CompareIntegers(Comparison::Equal));
        self.emit(Instruction::JumpIfFalse(differs));
        let index = self.counter();
        let walk = self.opening_a_walk(index, ONE);
        self.element_of(ONE, index);
        self.element_of(OTHER, index);
        self.through(prelude::EQ);
        self.emit(Instruction::JumpIfFalse(differs));
        self.closing_the_walk(index, &walk);
        self.emit(Instruction::Boolean(true));
        let end = self.label();
        self.emit(Instruction::Jump(end));
        self.emit(Instruction::Label(differs));
        self.emit(Instruction::Boolean(false));
        self.emit(Instruction::Label(end));
    }

    /// `Ord`: the first element that differs decides, and a prefix is below what holds more.
    ///
    /// Two elements are in order when the first is below the second, and out of order when the
    /// second is below the first. Neither means they are the same, and the walk goes on.
    fn is_less(&mut self) {
        let index = self.counter();
        let below = self.label();
        let above = self.label();
        let the_other_way = self.label();
        let alike = self.label();
        let walk = self.opening_a_walk(index, ONE);
        self.below_the_size(index, OTHER, walk.done);
        self.in_order(ONE, index);
        self.emit(Instruction::JumpIfFalse(the_other_way));
        self.emit(Instruction::Jump(below));
        self.emit(Instruction::Label(the_other_way));
        self.in_order(OTHER, index);
        self.emit(Instruction::JumpIfFalse(alike));
        self.emit(Instruction::Jump(above));
        self.emit(Instruction::Label(alike));
        self.closing_the_walk(index, &walk);
        self.size_of(ONE);
        self.size_of(OTHER);
        self.emit(Instruction::CompareIntegers(Comparison::Less));
        self.answering_after(below, above);
    }

    /// `Hash`: the textbook polynomial hash, over the hash of every element in turn.
    fn hashed(&mut self) {
        let code = self.temporary(&Descriptor::Long);
        self.emit(Instruction::Long(1));
        self.stored(code, &Descriptor::Long);
        let index = self.counter();
        let walk = self.opening_a_walk(index, ONE);
        self.loaded(code, &Descriptor::Long);
        self.emit(Instruction::Long(MULTIPLIER));
        self.emit(Instruction::Arithmetic(Arithmetic::Multiply));
        self.element_of(ONE, index);
        self.through(prelude::HASH);
        self.emit(Instruction::Arithmetic(Arithmetic::Add));
        self.stored(code, &Descriptor::Long);
        self.closing_the_walk(index, &walk);
        self.loaded(code, &Descriptor::Long);
    }

    /// `Show`: every element, shown by the instance of its own type, inside brackets.
    fn shown(&mut self) {
        let built = self.temporary(&text());
        self.emit(Instruction::Text(OPENING.to_owned()));
        self.stored(built, &text());
        let index = self.counter();
        let walk = self.opening_a_walk(index, ONE);
        self.separated(built, index);
        self.loaded(built, &text());
        self.element_of(ONE, index);
        self.through(prelude::SHOW);
        self.emit(Instruction::Concat);
        self.stored(built, &text());
        self.closing_the_walk(index, &walk);
        self.loaded(built, &text());
        self.emit(Instruction::Text(CLOSING.to_owned()));
        self.emit(Instruction::Concat);
    }

    /// Writes what stands between two elements, which the first element has nothing before it.
    fn separated(&mut self, built: u16, index: u16) {
        let first = self.label();
        self.loaded(index, &Descriptor::Integer);
        self.emit(Instruction::Integer(0));
        self.emit(Instruction::CompareIntegers(Comparison::Greater));
        self.emit(Instruction::JumpIfFalse(first));
        self.loaded(built, &text());
        self.emit(Instruction::Text(BETWEEN.to_owned()));
        self.emit(Instruction::Concat);
        self.stored(built, &text());
        self.emit(Instruction::Label(first));
    }

    /// Whether the element of `first` at the counter is below the other list's element there.
    fn in_order(&mut self, first: u16, index: u16) {
        self.element_of(first, index);
        self.element_of(the_other(first), index);
        self.through(prelude::ORD);
    }

    /// A walk over one list, opened with the test that ends it.
    fn opening_a_walk(&mut self, index: u16, list: u16) -> Walk {
        let walk = Walk {
            again: self.label(),
            done: self.label(),
        };
        self.emit(Instruction::Label(walk.again));
        self.below_the_size(index, list, walk.done);
        walk
    }

    /// The end of one turn of a walk, which counts on and goes round again.
    fn closing_the_walk(&mut self, index: u16, walk: &Walk) {
        self.emit(Instruction::Increment { slot: index });
        self.emit(Instruction::Jump(walk.again));
        self.emit(Instruction::Label(walk.done));
    }

    /// Jumps to `done` unless the counter is still inside the list.
    fn below_the_size(&mut self, index: u16, list: u16, done: Label) {
        self.loaded(index, &Descriptor::Integer);
        self.size_of(list);
        self.emit(Instruction::CompareIntegers(Comparison::Less));
        self.emit(Instruction::JumpIfFalse(done));
    }

    /// How many values the list in `list` holds.
    fn size_of(&mut self, list: u16) {
        self.loaded(list, &Descriptor::reference(LIST));
        self.emit(Instruction::InvokeInterface(reaching(
            "size",
            Vec::new(),
            Descriptor::Integer,
        )));
    }

    /// The element the list in `list` holds at the counter, read back as the type it is.
    fn element_of(&mut self, list: u16, index: u16) {
        self.loaded(list, &Descriptor::reference(LIST));
        self.loaded(index, &Descriptor::Integer);
        self.emit(Instruction::InvokeInterface(reaching(
            "get",
            vec![Descriptor::Integer],
            object(),
        )));
        for instruction in read_back_as(&self.held) {
            self.emit(instruction);
        }
    }

    /// Hands what is on the stack to the instance of `of` at the type the list holds.
    ///
    /// A type whose instance a module wrote or derived is a call of that instance's method, and
    /// one of the types the JVM holds is what that instance amounts to, written out here.
    fn through(&mut self, of: &str) {
        let method = prelude::method_of(of).expect("a standard trait declares one method");
        let Some(instance) = self.lowering.answering(method, &self.over.element) else {
            for instruction in supplied(of, &self.held) {
                self.emit(instruction);
            }
            return;
        };
        self.emit(instance.called());
    }

    /// The two answers a walk jumped out of, after the answer the walk itself worked out.
    fn answering_after(&mut self, yes: Label, no: Label) {
        let end = self.label();
        self.emit(Instruction::Jump(end));
        self.emit(Instruction::Label(yes));
        self.emit(Instruction::Boolean(true));
        self.emit(Instruction::Jump(end));
        self.emit(Instruction::Label(no));
        self.emit(Instruction::Boolean(false));
        self.emit(Instruction::Label(end));
    }

    /// A local holding the whole number the walk counts with, which starts at nothing.
    fn counter(&mut self) -> u16 {
        let at = self.temporary(&Descriptor::Integer);
        self.emit(Instruction::Integer(0));
        self.stored(at, &Descriptor::Integer);
        at
    }

    fn loaded(&mut self, slot: u16, of: &Descriptor) {
        self.emit(Instruction::Load {
            slot,
            of: of.clone(),
        });
    }

    fn stored(&mut self, slot: u16, of: &Descriptor) {
        self.emit(Instruction::Store {
            slot,
            of: of.clone(),
        });
    }

    fn temporary(&mut self, of: &Descriptor) -> u16 {
        let at = self.next_slot;
        self.next_slot += of.width();
        self.locals += of.width();
        at
    }

    fn label(&mut self) -> Label {
        let label = Label(self.next_label);
        self.next_label += 1;
        label
    }

    fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
}

/// The two places one turn of a walk over a list jumps to.
struct Walk {
    again: Label,
    done: Label,
}

/// What the prelude's own instance of `of` amounts to, over an element the JVM holds as `held`.
fn supplied(of: &str, held: &Descriptor) -> Vec<Instruction> {
    match of {
        prelude::EQ => compared(Some(held), Comparison::Equal),
        prelude::ORD => compared(Some(held), Comparison::Less),
        prelude::HASH => hashed_as(held),
        prelude::SHOW => shown_as(held),
        of => unreachable!("`{of}` is no trait the prelude writes an instance over `List` of"),
    }
}

/// What the JVM carries one element of the list as, which is what reading one back gives.
fn carried_by(lowering: &Lowering<'_>, over: &OverAList) -> Descriptor {
    lowering
        .shapes
        .carried(&over.element)
        .expect("a list whose elements answer a standard trait holds what something carries")
}

/// The slot the body's own locals begin at, which is after every parameter it is handed.
fn taken_by(signature: &Signature) -> u16 {
    signature
        .parameters
        .iter()
        .flatten()
        .map(Descriptor::width)
        .sum()
}

/// The text a shown list is built in, which is the class a JVM holds a string in.
fn text() -> Descriptor {
    Descriptor::reference("java/lang/String")
}

/// The one of the two lists the method is handed that is not `list`.
const fn the_other(list: u16) -> u16 {
    if list == ONE { OTHER } else { ONE }
}

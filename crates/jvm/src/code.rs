//! One method body, as the bytes a `Code` attribute holds.

use std::collections::HashMap;

use lumen_ir::{Body, ClassName, Descriptor, Instruction, Label, MethodDescriptor};

use crate::bytes::Bytes;
use crate::frame::{Frame, Held, Hierarchy};
use crate::opcode;
use crate::pool::Pool;

/// A body's bytes, with everything the `Code` attribute is written from.
pub(crate) struct Assembled {
    pub(crate) code: Vec<u8>,
    pub(crate) max_stack: u16,
    pub(crate) max_locals: u16,
    /// Each place a jump lands, with what holds there, in the order they are written.
    pub(crate) frames: Vec<(u16, Frame)>,
}

/// Assembles `body` into the bytes a method of `descriptor` runs.
pub(crate) fn assemble(
    body: &Body,
    descriptor: &MethodDescriptor,
    receiver: Option<&ClassName>,
    context: &mut Context<'_>,
) -> Assembled {
    let slots = descriptor.width() + body.locals + u16::from(receiver.is_some());
    let mut assembling = Assembling {
        bytes: Bytes::default(),
        frame: Frame::entering(&descriptor.parameters, receiver, slots),
        reachable: true,
        landings: HashMap::new(),
        expected: HashMap::new(),
        pending: Vec::new(),
        deepest: 0,
        next: u32::MAX,
    };
    for instruction in &body.instructions {
        assembling.one(instruction, context);
    }
    assembling.finish(slots)
}

/// What assembling one body needs from the class around it.
pub(crate) struct Context<'a> {
    pub(crate) pool: &'a mut Pool,
    pub(crate) hierarchy: &'a Hierarchy,
}

/// A body part-way through being assembled.
pub(crate) struct Assembling {
    bytes: Bytes,
    frame: Frame,
    /// Whether anything reaches the instruction about to be written.
    reachable: bool,
    landings: HashMap<Label, u16>,
    expected: HashMap<Label, Frame>,
    /// Each branch written before its landing was known.
    pending: Vec<Patch>,
    deepest: usize,
    /// The number the next label the assembler makes for itself takes.
    next: u32,
}

impl Assembling {
    fn one(&mut self, instruction: &Instruction, context: &mut Context<'_>) {
        if let Instruction::Label(label) = instruction {
            self.landing(*label, context);
            return;
        }
        if !self.reachable {
            return;
        }
        self.write(instruction, context);
    }

    fn finish(mut self, slots: u16) -> Assembled {
        for patch in &self.pending {
            let landing = self.landings.get(&patch.label).copied().unwrap_or_default();
            let away = i64::from(landing) - i64::from(patch.from);
            // A body too large for a branch to reach across is written out of range on purpose:
            // the JVM refuses the class, which says so where a truncated offset would not.
            let away = i16::try_from(away).unwrap_or(i16::MAX);
            self.bytes.patch_u2(patch.at, away.cast_unsigned());
        }
        let frames = self.written_frames();
        Assembled {
            code: self.bytes.taken(),
            max_stack: u16::try_from(self.deepest).unwrap_or(u16::MAX),
            max_locals: slots,
            frames,
        }
    }

    /// Each place a jump lands, in the order they are written, with no offset named twice.
    fn written_frames(&self) -> Vec<(u16, Frame)> {
        let mut landings: Vec<(&Label, &u16)> = self.landings.iter().collect();
        landings.sort_by_key(|(label, at)| (**at, **label));
        let mut frames: Vec<(u16, Frame)> = Vec::new();
        for (label, at) in landings {
            if frames.last().is_some_and(|(seen, _)| *seen == *at) {
                continue;
            }
            if let Some(frame) = self.expected.get(label) {
                frames.push((*at, frame.clone()));
            }
        }
        frames
    }

    /// Writes a comparison as the truth value it leaves behind.
    pub(crate) fn truth(&mut self, opcode: u8, consumes: usize, context: &mut Context<'_>) {
        let yes = self.own_label();
        let done = self.own_label();
        for _ in 0..consumes {
            self.pop();
        }
        self.branch(opcode, yes, context);
        self.byte(opcode::ICONST_0);
        self.push(Held::Integer);
        self.branch(opcode::GOTO, done, context);
        self.unreachable();
        self.landing(yes, context);
        self.byte(opcode::ICONST_1);
        self.push(Held::Integer);
        self.landing(done, context);
    }

    /// Records where a label lands, and what holds there.
    pub(crate) fn landing(&mut self, label: Label, context: &mut Context<'_>) {
        if self.reachable {
            self.expect(label, context);
        }
        let Some(frame) = self.expected.get(&label) else {
            return;
        };
        self.frame = frame.clone();
        self.reachable = true;
        let at = u16::try_from(self.bytes.len()).unwrap_or(u16::MAX);
        self.landings.insert(label, at);
    }

    /// A label of the assembler's own, for the branches one instruction is made of.
    fn own_label(&mut self) -> Label {
        self.next -= 1;
        Label(self.next)
    }

    pub(crate) fn push(&mut self, held: Held) {
        self.frame.stack.push(held);
        self.deepest = self.deepest.max(self.frame.depth());
    }

    pub(crate) fn pop(&mut self) -> Option<Held> {
        self.frame.stack.pop()
    }

    pub(crate) fn top(&self) -> Option<Held> {
        self.frame.stack.last().cloned()
    }

    pub(crate) fn hold(&mut self, slot: u16, of: &Descriptor) {
        self.frame.hold(slot, of);
    }

    /// Every copy of the instance made at `at` now holds a built one.
    pub(crate) fn initialised(&mut self, at: u16, class: &ClassName) {
        self.frame.initialised(at, class);
    }

    pub(crate) fn byte(&mut self, value: u8) {
        self.bytes.u1(value);
    }

    pub(crate) fn short(&mut self, value: u16) {
        self.bytes.u2(value);
    }

    /// Nothing reaches what comes next, so it is left out until a label says otherwise.
    pub(crate) fn unreachable(&mut self) {
        self.reachable = false;
    }

    /// An instruction naming a local, widened when the slot does not fit in one byte.
    pub(crate) fn indexed(&mut self, opcode: u8, slot: u16) {
        if let Ok(narrow) = u8::try_from(slot) {
            self.bytes.u1(opcode);
            self.bytes.u1(narrow);
        } else {
            self.bytes.u1(opcode::WIDE);
            self.bytes.u1(opcode);
            self.bytes.u2(slot);
        }
    }

    pub(crate) fn branch(&mut self, opcode: u8, label: Label, context: &mut Context<'_>) {
        self.expect(label, context);
        let from = self.offset();
        self.bytes.u1(opcode);
        self.pending.push(Patch {
            at: self.bytes.len(),
            label,
            from,
        });
        self.bytes.u2(0);
    }

    /// Says that control may reach `label` holding what it holds now.
    fn expect(&mut self, label: Label, context: &mut Context<'_>) {
        let merged = match self.expected.get(&label) {
            Some(known) => known.merged_with(&self.frame, context.hierarchy),
            None => self.frame.clone(),
        };
        self.expected.insert(label, merged);
    }

    /// The offset the next instruction is written at.
    pub(crate) fn offset(&self) -> u16 {
        u16::try_from(self.bytes.len()).unwrap_or(u16::MAX)
    }
}

/// A branch whose landing was not known when it was written.
struct Patch {
    at: usize,
    label: Label,
    /// The offset the branch is measured from, which is where its opcode sits.
    from: u16,
}

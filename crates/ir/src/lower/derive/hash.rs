//! The body of `Hash$T$hashed`, which works a whole number out of what a value holds.
//!
//! `docs/specs/derive.md` works it out from the tag of the variant the value is and the hash of
//! each value it carries, in the order it carries them, so equal values hash alike. Which number
//! two unequal values work out to is not part of that spec, and nothing may depend on it.

use lumen_ast::{TypeDeclaration, Variant};
use lumen_resolver::prelude;

use crate::code::{Arithmetic, Instruction, Label};
use crate::descriptor::Descriptor;
use crate::lower::derive::{Holds, ONE, Writing, record_of, variants_of};

/// What each value already worked out is multiplied by before the next is added to it.
///
/// It is the odd number every hash of a sequence is spread by, so two values holding the same
/// two things the other way round rarely work out to one number.
const SPREAD: i64 = 31;

/// Writes the body: a number to start from, whatever the type holds, and the answer.
pub(super) fn hashed(writing: &mut Writing<'_>, declared: &TypeDeclaration) {
    if let Some(fields) = record_of(declared) {
        writing.emit(Instruction::Long(0));
        for holds in writing.record_holds(&declared.name.text, fields) {
            taking_in(writing, &holds);
        }
    }
    if let Some(variants) = variants_of(declared) {
        by_variant(writing, variants);
    }
    writing.emit(Instruction::Return(Some(Descriptor::Long)));
}

/// Variants: the tag to start from, and then whatever that one variant carries.
fn by_variant(writing: &mut Writing<'_>, variants: &[Variant]) {
    let end = writing.label();
    let base = writing.base_of(variants);
    writing.tag(&base, ONE);
    writing.emit(Instruction::Widen);
    for variant in variants {
        carried_by(writing, variant, end);
    }
    writing.emit(Instruction::Label(end));
}

/// What one variant carries, taken in where the value turns out to be that variant.
fn carried_by(writing: &mut Writing<'_>, variant: &Variant, end: Label) {
    let held = writing.variant_holds(variant);
    if held.is_empty() {
        return;
    }
    let next = writing.label();
    let shape = writing.shape_of(variant);
    writing.unless_it_is(&shape, next);
    for holds in &held {
        taking_in(writing, holds);
    }
    writing.emit(Instruction::Jump(end));
    writing.emit(Instruction::Label(next));
}

/// One value the type holds, taken into the number already worked out below it on the stack.
fn taking_in(writing: &mut Writing<'_>, holds: &Holds<'_>) {
    writing.emit(Instruction::Long(SPREAD));
    writing.emit(Instruction::Arithmetic(Arithmetic::Multiply));
    writing.read(holds, ONE);
    writing.through(prelude::HASH, holds);
    writing.emit(Instruction::Arithmetic(Arithmetic::Add));
}

//! The body of `Eq$T$is_equal`, which compares two values by their state.
//!
//! `docs/specs/derive.md` has two values equal when they are the same variant and everything they
//! hold is equal, and the walk stops at the first value that differs, as `&&` stops there.

use lumen_ast::{TypeDeclaration, Variant};
use lumen_resolver::prelude;

use crate::code::{Comparison, Instruction, Label};
use crate::descriptor::Descriptor;
use crate::lower::derive::{Holds, ONE, OTHER, Writing, record_of, variants_of};

/// Writes the body: whatever the type holds, and then the two answers.
pub(super) fn is_equal(writing: &mut Writing<'_>, declared: &TypeDeclaration) {
    let differs = writing.label();
    if let Some(fields) = record_of(declared) {
        for holds in writing.record_holds(&declared.name.text, fields) {
            equal(writing, &holds, differs);
        }
    }
    if let Some(variants) = variants_of(declared) {
        same_variant(writing, variants, differs);
    }
    answer(writing, differs);
}

/// Variants: the same variant first, and then whatever that variant carries.
///
/// The tags settle which variant both values are, so the walk that follows reaches the one block
/// that carries anything and jumps over every other.
fn same_variant(writing: &mut Writing<'_>, variants: &[Variant], differs: Label) {
    let equally = writing.label();
    let base = writing.base_of(variants);
    writing.tag(&base, ONE);
    writing.tag(&base, OTHER);
    writing.emit(Instruction::CompareIntegers(Comparison::Equal));
    writing.emit(Instruction::JumpIfFalse(differs));
    for variant in variants {
        let held = writing.variant_holds(variant);
        if held.is_empty() {
            continue;
        }
        let next = writing.label();
        let shape = writing.shape_of(variant);
        writing.unless_it_is(&shape, next);
        for holds in &held {
            equal(writing, holds, differs);
        }
        writing.emit(Instruction::Jump(equally));
        writing.emit(Instruction::Label(next));
    }
    writing.emit(Instruction::Label(equally));
}

/// One value both sides hold, which leaves for `differs` unless the two of it are equal.
fn equal(writing: &mut Writing<'_>, holds: &Holds<'_>, differs: Label) {
    writing.read(holds, ONE);
    writing.read(holds, OTHER);
    writing.through(prelude::EQ, holds);
    writing.emit(Instruction::JumpIfFalse(differs));
}

/// The two answers and the one way out, which every comparison above has jumped into.
fn answer(writing: &mut Writing<'_>, differs: Label) {
    let end = writing.label();
    writing.emit(Instruction::Boolean(true));
    writing.emit(Instruction::Jump(end));
    writing.emit(Instruction::Label(differs));
    writing.emit(Instruction::Boolean(false));
    writing.emit(Instruction::Label(end));
    writing.emit(Instruction::Return(Some(Descriptor::Boolean)));
}

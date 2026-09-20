//! The body of `Ord$T$is_less`, which says which of two values comes first.
//!
//! `docs/specs/derive.md` orders a variant by the order the type declares it in and two values of
//! one variant by what they carry, the first value that differs deciding. Only `Ord` is asked of
//! what a type holds: a value that comes neither before the other nor after it is the one the walk
//! carries on past.

use lumen_ast::{TypeDeclaration, Variant};
use lumen_resolver::prelude;

use crate::code::{Comparison, Instruction, Label};
use crate::descriptor::Descriptor;
use crate::lower::derive::{Holds, ONE, OTHER, Writing, record_of, variants_of};

/// Where the walk leaves for once it has settled which of the two values comes first.
struct Settled {
    less: Label,
    not_less: Label,
}

/// Writes the body: whatever the type holds, and then the two answers.
pub(super) fn is_less(writing: &mut Writing<'_>, declared: &TypeDeclaration) {
    let settled = Settled {
        less: writing.label(),
        not_less: writing.label(),
    };
    if let Some(fields) = record_of(declared) {
        for holds in writing.record_holds(&declared.name.text, fields) {
            deciding(writing, &holds, &settled);
        }
    }
    if let Some(variants) = variants_of(declared) {
        by_variant(writing, variants, &settled);
    }
    answer(writing, &settled);
}

/// Variants: which variant each value is first, and then what that variant carries.
///
/// A variant the type declares first comes first, so the tags decide on their own unless both
/// values are the same variant, which is the one case the payload is reached in.
fn by_variant(writing: &mut Writing<'_>, variants: &[Variant], settled: &Settled) {
    let base = writing.base_of(variants);
    writing.tag(&base, ONE);
    writing.tag(&base, OTHER);
    writing.emit(Instruction::CompareIntegers(Comparison::Less));
    leaving_where_it_holds(writing, settled.less);
    writing.tag(&base, ONE);
    writing.tag(&base, OTHER);
    writing.emit(Instruction::CompareIntegers(Comparison::Equal));
    writing.emit(Instruction::JumpIfFalse(settled.not_less));
    for variant in variants {
        let held = writing.variant_holds(variant);
        if held.is_empty() {
            continue;
        }
        let next = writing.label();
        let shape = writing.shape_of(variant);
        writing.unless_it_is(&shape, next);
        for holds in &held {
            deciding(writing, holds, settled);
        }
        writing.emit(Instruction::Jump(settled.not_less));
        writing.emit(Instruction::Label(next));
    }
}

/// One value both sides hold, which decides unless the two of it come in neither order.
fn deciding(writing: &mut Writing<'_>, holds: &Holds<'_>, settled: &Settled) {
    writing.read(holds, ONE);
    writing.read(holds, OTHER);
    writing.through(prelude::ORD, holds);
    leaving_where_it_holds(writing, settled.less);
    writing.read(holds, OTHER);
    writing.read(holds, ONE);
    writing.through(prelude::ORD, holds);
    leaving_where_it_holds(writing, settled.not_less);
}

/// Leaves for `answer` where the truth value on the stack holds, and carries on where it does not.
///
/// There is no jump on a truth value that holds, so the value is turned into the other one and
/// the jump that leaves on a false one is the jump that leaves on a true one.
fn leaving_where_it_holds(writing: &mut Writing<'_>, answer: Label) {
    writing.emit(Instruction::Not);
    writing.emit(Instruction::JumpIfFalse(answer));
}

/// The two answers and the one way out, which every decision above has jumped into.
fn answer(writing: &mut Writing<'_>, settled: &Settled) {
    let end = writing.label();
    writing.emit(Instruction::Jump(settled.not_less));
    writing.emit(Instruction::Label(settled.less));
    writing.emit(Instruction::Boolean(true));
    writing.emit(Instruction::Jump(end));
    writing.emit(Instruction::Label(settled.not_less));
    writing.emit(Instruction::Boolean(false));
    writing.emit(Instruction::Label(end));
    writing.emit(Instruction::Return(Some(Descriptor::Boolean)));
}

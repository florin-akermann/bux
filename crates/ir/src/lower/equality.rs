//! What makes two values the same value, which is one answer for `==` and for a literal pattern.
//!
//! `==` is `Eq`, and version 0.1 ships `Int`, `Bool`, and `String`, so this settles those three
//! and nothing else. Two whole numbers or two truth values are the same when the JVM says they
//! are, and two strings are the same when they hold the same characters rather than when they are
//! one object; identity is never what Lumen asks about.

use crate::code::{Comparison, Instruction, MethodRef};
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
use crate::lower::shape::object;

/// What decides whether two values above it on the stack are the same, or are not.
///
/// A value carried by nothing has nothing on the stack, and there is only one of it, so the
/// answer is settled before anything runs.
pub(crate) fn compared(held: Option<&Descriptor>, how: Comparison) -> Vec<Instruction> {
    match held {
        None => vec![Instruction::Boolean(how == Comparison::Equal)],
        Some(Descriptor::Long) => vec![Instruction::CompareLongs(how)],
        Some(Descriptor::Boolean | Descriptor::Integer) => vec![Instruction::CompareIntegers(how)],
        Some(Descriptor::Reference(class)) if class == &string_class() => by_value(how),
        Some(held) => {
            unreachable!("`{held}` has no `Eq`, so inference refused the comparison of two of them")
        }
    }
}

/// The one class a comparison of references is ever between, which is the one that has `Eq`.
fn string_class() -> ClassName {
    ClassName::new("java/lang/String")
}

/// Two strings, compared by the characters they hold rather than by being one object.
///
/// The call is on `String` itself and not on `Object`, so a reference of any other class reaching
/// here writes a class file the verifier refuses. The identity `Object.equals` would answer with
/// is then unreachable by construction rather than by the type checker alone.
fn by_value(how: Comparison) -> Vec<Instruction> {
    let mut instructions = vec![Instruction::InvokeVirtual(MethodRef {
        class: string_class(),
        name: "equals".to_owned(),
        descriptor: MethodDescriptor::new(vec![object()], Some(Descriptor::Boolean)),
    })];
    if how != Comparison::Equal {
        instructions.push(Instruction::Not);
    }
    instructions
}

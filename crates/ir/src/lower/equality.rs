//! What makes two values the same value, which is one answer for `==`, `match`, and `equals`.
//!
//! Two whole numbers or two truth values are the same when the JVM says they are. Two references
//! are the same when what they hold is the same, so `User { id: 1 }` equals another one built the
//! same way; identity would make two of one record different, which Lumen never means.

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
        Some(Descriptor::Reference(_)) => by_value(how),
    }
}

/// Two references, compared by what they hold, which is what `Objects.equals` does.
fn by_value(how: Comparison) -> Vec<Instruction> {
    let mut instructions = vec![Instruction::InvokeStatic(MethodRef {
        class: ClassName::new("java/util/Objects"),
        name: "equals".to_owned(),
        descriptor: MethodDescriptor::new(vec![object(), object()], Some(Descriptor::Boolean)),
    })];
    if how != Comparison::Equal {
        instructions.push(Instruction::Not);
    }
    instructions
}

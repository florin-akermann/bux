//! What the prelude's instances of the four standard traits amount to, written out where they
//! are called.
//!
//! The prelude has `Eq`, `Ord`, `Hash`, and `Show` for `Bool`, `Int`, and `String`, which
//! `docs/specs/traits.md` writes out, and `library/prelude.lm` writes all but `Hash<String>`.
//! None of them is called: a use is what the instance amounts to, written out here as the
//! instructions it always was, which is what `docs/specs/library.md` says the library is for.
//! Two whole numbers or two truth values stand in the order the JVM puts them, and two strings
//! are read by the characters they hold rather than by being one object; identity is never what
//! Lumen asks about.

use crate::code::{Comparison, Instruction, MethodRef};
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
use crate::lower::shape::object;

/// What decides whether two values above it on the stack are the same, or are in order.
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

/// The whole number the value above it on the stack stands for, which is `Hash`'s one method.
///
/// `Int` stands for itself, a truth value for the `0` or the `1` a JVM already holds it as, and a
/// string for the number its characters work out to. Each is widened because `hashed` gives an
/// `Int`, which is a whole number the JVM holds as a long.
pub(crate) fn hashed_as(held: &Descriptor) -> Vec<Instruction> {
    match held {
        Descriptor::Long => Vec::new(),
        Descriptor::Boolean | Descriptor::Integer => vec![Instruction::Widen],
        Descriptor::Reference(class) if class == &string_class() => vec![
            Instruction::InvokeVirtual(MethodRef {
                class: string_class(),
                name: "hashCode".to_owned(),
                descriptor: MethodDescriptor::new(Vec::new(), Some(Descriptor::Integer)),
            }),
            Instruction::Widen,
        ],
        held => unreachable!("`{held}` has no `Hash`, so inference refused a hash of one"),
    }
}

/// The text the value above it on the stack is shown as, which is `Show`'s one method.
///
/// A string is shown as the characters it holds and nothing more, so it is already its own text;
/// the other two are rendered by the JVM, which writes the digits and `true` or `false`.
pub(crate) fn shown_as(held: &Descriptor) -> Vec<Instruction> {
    match held {
        Descriptor::Reference(class) if class == &string_class() => Vec::new(),
        Descriptor::Long => vec![rendered("java/lang/Long", Descriptor::Long)],
        Descriptor::Boolean => vec![rendered("java/lang/Boolean", Descriptor::Boolean)],
        held => unreachable!("`{held}` has no `Show`, so inference refused showing one"),
    }
}

/// The JVM's own `toString` for the one value above it on the stack, at the class that holds it.
///
/// It is the static one taking the value rather than the virtual one every object is born with,
/// so nothing here reads a `toString` a Lumen type never asked for.
fn rendered(class: &str, of: Descriptor) -> Instruction {
    Instruction::InvokeStatic(MethodRef {
        class: ClassName::new(class),
        name: "toString".to_owned(),
        descriptor: MethodDescriptor::new(
            vec![of],
            Some(Descriptor::reference("java/lang/String")),
        ),
    })
}

/// Two strings, read by the characters they hold rather than by being one object.
///
/// The call is on `String` itself and not on `Object`, so a reference of any other class reaching
/// here writes a class file the verifier refuses. The identity `Object.equals` would answer with
/// is then unreachable by construction rather than by the type checker alone.
///
/// `==` and `!=` ask `equals`, and the four comparisons ask `compareTo`, whose answer stands
/// against `0` the way the comparison stands: `docs/specs/traits.md` orders two strings by their
/// characters, which is the order `compareTo` reports.
fn by_value(how: Comparison) -> Vec<Instruction> {
    match how {
        Comparison::Equal => vec![equals()],
        Comparison::NotEqual => vec![equals(), Instruction::Not],
        ordered => vec![
            Instruction::InvokeVirtual(MethodRef {
                class: string_class(),
                name: "compareTo".to_owned(),
                descriptor: MethodDescriptor::new(
                    vec![Descriptor::reference("java/lang/String")],
                    Some(Descriptor::Integer),
                ),
            }),
            Instruction::Integer(0),
            Instruction::CompareIntegers(ordered),
        ],
    }
}

/// Whether the two strings on the stack hold the same characters.
fn equals() -> Instruction {
    Instruction::InvokeVirtual(MethodRef {
        class: string_class(),
        name: "equals".to_owned(),
        descriptor: MethodDescriptor::new(vec![object()], Some(Descriptor::Boolean)),
    })
}

/// The one class a comparison of references is ever between, which is the one that has `Eq`.
fn string_class() -> ClassName {
    ClassName::new("java/lang/String")
}

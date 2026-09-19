//! Lowering: a checked module as the classes a JVM loads.
//!
//! The phase consumes the typed tree and yields a [`Lowered`] program, which says what classes to
//! write and what their methods do, in terms the JVM understands and Lumen never mentions.
//! `docs/specs/codegen.md` is the specification. Writing the bytes is `lumen-jvm`'s work; nothing
//! here knows the layout of a class file.

mod class;
mod code;
mod descriptor;

pub use crate::class::{Class, Extending, Field, Method, Reached};
pub use crate::code::{Arithmetic, Body, Comparison, FieldRef, Instruction, Label, MethodRef};
pub use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};

/// Every class a module becomes, in the order they are written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lowered {
    pub classes: Vec<Class>,
}

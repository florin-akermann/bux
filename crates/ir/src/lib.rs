//! Lowering: a checked module as the classes a JVM loads.
//!
//! The phase consumes the typed tree and yields a [`Lowered`] program, which says what classes to
//! write and what their methods do, in terms the JVM understands and Lumen never mentions.
//! `docs/specs/codegen.md` is the specification. Writing the bytes is `lumen-jvm`'s work; nothing
//! here knows the layout of a class file.

mod asked;
mod class;
mod code;
mod descriptor;
mod lower;

pub use crate::asked::Asked;
pub use crate::class::{Class, Extending, Field, Method, Reached};
pub use crate::code::{
    Arithmetic, Body, Comparison, FieldRef, Guard, Instruction, Label, MethodRef,
};
pub use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
pub use crate::lower::{THE_ONE_SHAPE, is_a_program, lower, lower_prelude};

/// Every class a module becomes, in the order they are written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lowered {
    pub classes: Vec<Class>,
    /// What this module asked the modules it imports for, which their own builds write.
    pub asks: Asked,
}

//! The classes a module becomes.

use crate::code::Body;
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};

/// Whether a class can be extended, which every class a module writes answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extending {
    /// A record, a variant, or a module: nothing extends it.
    Never,
    /// The base of an algebraic data type, which only its own variants extend.
    ByItsVariants,
}

/// Whether a method is reached through an instance or through the class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reached {
    /// A function of the module, or a constructor of a boxed value.
    ThroughTheClass,
    /// A constructor, or a method of an instance.
    ThroughAnInstance,
}

/// One field of one class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub of: Descriptor,
}

/// One method of one class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Method {
    pub name: String,
    pub descriptor: MethodDescriptor,
    pub reached: Reached,
    pub body: Body,
}

/// One class a module becomes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Class {
    pub name: ClassName,
    pub extends: ClassName,
    pub extending: Extending,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
}

impl Class {
    /// A class extending `java.lang.Object` that nothing extends.
    #[must_use]
    pub fn new(name: ClassName) -> Self {
        Self {
            name,
            extends: ClassName::new("java/lang/Object"),
            extending: Extending::Never,
            fields: Vec::new(),
            methods: Vec::new(),
        }
    }
}

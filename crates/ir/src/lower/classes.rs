//! The classes a module's type declarations become.
//!
//! A record is one class holding its fields; an algebraic data type is a base holding the tag
//! and one class per variant holding what that variant carries. A class declares its constructor
//! and nothing else, because `==` is `Eq` and version 0.1 gives no declared type an instance.

use crate::class::{Class, Extending, Field, Method, Reached};
use crate::code::{Body, FieldRef, Instruction, MethodRef};
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
use crate::lower::shape::{CONSTRUCTOR, Declared, Shape, Shapes, TAG};

impl Shapes {
    /// Every class the declarations become, in the order the module declares them.
    pub(crate) fn classes(&self) -> Vec<Class> {
        let mut classes = Vec::new();
        for declaration in self.declarations() {
            match declaration {
                Declared::Record(constructor) => {
                    classes.push(record_class(self.built(constructor)));
                }
                Declared::Variants { base, constructors } => {
                    classes.push(base_class(base));
                    for constructor in constructors {
                        classes.push(variant_class(self.built(constructor), base));
                    }
                }
            }
        }
        classes
    }
}

/// A record: one class holding one field per field the type declares, and nothing extends it.
pub(crate) fn record_class(shape: &Shape) -> Class {
    let mut class = Class::new(shape.class.clone());
    class.fields = held(shape);
    class.methods = vec![constructor(shape, &object_class())];
    class
}

/// The base of an algebraic data type: the tag, and nothing but its own variants extending it.
fn base_class(base: &ClassName) -> Class {
    let mut class = Class::new(base.clone());
    class.extending = Extending::ByItsVariants;
    class.fields = vec![Field {
        name: TAG.to_owned(),
        of: Descriptor::Integer,
    }];
    class.methods = vec![base_constructor(base)];
    class
}

/// One variant: the values it carries, and the tag it hands its base.
fn variant_class(shape: &Shape, base: &ClassName) -> Class {
    let mut class = Class::new(shape.class.clone());
    class.extends = base.clone();
    class.fields = held(shape);
    class.methods = vec![constructor(shape, base)];
    class
}

/// The constructor of a class that holds values: it fills its fields, then hands its tag up.
///
/// A value class writes its fields before its base is constructed, which is the order the JVM
/// asks of a strict field, so the value is whole by the time anything above it runs.
fn constructor(shape: &Shape, extends: &ClassName) -> Method {
    let this = Descriptor::Reference(shape.class.clone());
    let mut instructions = Vec::new();
    let mut slot = 1;
    for field in held(shape) {
        instructions.push(Instruction::Load {
            slot: 0,
            of: this.clone(),
        });
        instructions.push(Instruction::Load {
            slot,
            of: field.of.clone(),
        });
        slot += field.of.width();
        instructions.push(Instruction::PutField(reached(shape, &field)));
    }
    instructions.push(Instruction::Load { slot: 0, of: this });
    if let Some(tag) = shape.tag {
        instructions.push(Instruction::Integer(tag));
    }
    instructions.push(Instruction::Construct(MethodRef {
        class: extends.clone(),
        name: CONSTRUCTOR.to_owned(),
        descriptor: MethodDescriptor::new(tag_taken(shape), None),
    }));
    instructions.push(Instruction::Return(None));
    instance_method(CONSTRUCTOR, shape.constructor(), instructions, 0)
}

/// The base's own constructor, which takes the tag, writes it, and is all a base does.
fn base_constructor(base: &ClassName) -> Method {
    let this = Descriptor::Reference(base.clone());
    let instructions = vec![
        Instruction::Load {
            slot: 0,
            of: this.clone(),
        },
        Instruction::Load {
            slot: 1,
            of: Descriptor::Integer,
        },
        Instruction::PutField(FieldRef {
            class: base.clone(),
            name: TAG.to_owned(),
            of: Descriptor::Integer,
        }),
        Instruction::Load { slot: 0, of: this },
        Instruction::Construct(MethodRef {
            class: object_class(),
            name: CONSTRUCTOR.to_owned(),
            descriptor: MethodDescriptor::new(Vec::new(), None),
        }),
        Instruction::Return(None),
    ];
    let descriptor = MethodDescriptor::new(vec![Descriptor::Integer], None);
    instance_method(CONSTRUCTOR, descriptor, instructions, 0)
}

/// A method reached through an instance, which every method a type declaration writes is.
fn instance_method(
    name: &str,
    descriptor: MethodDescriptor,
    instructions: Vec<Instruction>,
    locals: u16,
) -> Method {
    Method {
        name: name.to_owned(),
        descriptor,
        reached: Reached::ThroughAnInstance,
        body: Body {
            instructions,
            locals,
            guards: Vec::new(),
        },
    }
}

/// The fields of the class, which are the values it holds that are carried by anything.
fn held(shape: &Shape) -> Vec<Field> {
    shape
        .carries
        .iter()
        .filter_map(|carried| {
            carried.of.clone().map(|of| Field {
                name: carried.name.clone(),
                of,
            })
        })
        .collect()
}

fn reached(shape: &Shape, field: &Field) -> FieldRef {
    FieldRef {
        class: shape.class.clone(),
        name: field.name.clone(),
        of: field.of.clone(),
    }
}

/// What a class hands its base, which is the tag when it has one and nothing when it has none.
fn tag_taken(shape: &Shape) -> Vec<Descriptor> {
    match shape.tag {
        Some(_) => vec![Descriptor::Integer],
        None => Vec::new(),
    }
}

fn object_class() -> ClassName {
    ClassName::new("java/lang/Object")
}

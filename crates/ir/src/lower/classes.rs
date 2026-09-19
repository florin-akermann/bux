//! The classes a module's type declarations become.
//!
//! A record is one class holding its fields; an algebraic data type is a base holding the tag
//! and one class per variant holding what that variant carries. Each of them is given the
//! `equals` that `==` calls, because two records that hold the same values are the same value.

use crate::class::{Class, Extending, Field, Method, Reached};
use crate::code::{Body, Comparison, FieldRef, Instruction, Label, MethodRef};
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
use crate::lower::equality::compared;
use crate::lower::shape::{CONSTRUCTOR, Declared, Shape, Shapes, TAG, object};

/// Where the `equals` a `==` of two references calls ends up, whatever the two are.
const EQUALS: &str = "equals";

/// The one place a comparison lands when it has found the two values to differ.
const DIFFERENT: Label = Label(0);

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
fn record_class(shape: &Shape) -> Class {
    let mut class = Class::new(shape.class.clone());
    class.fields = held(shape);
    class.methods = vec![constructor(shape, &object_class()), equals(shape)];
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
    class.methods = vec![constructor(shape, base), equals(shape)];
    class
}

/// The constructor of a class that holds values: it hands its tag up, then fills its fields.
fn constructor(shape: &Shape, extends: &ClassName) -> Method {
    let this = Descriptor::Reference(shape.class.clone());
    let mut instructions = vec![Instruction::Load {
        slot: 0,
        of: this.clone(),
    }];
    if let Some(tag) = shape.tag {
        instructions.push(Instruction::Integer(tag));
    }
    instructions.push(Instruction::Construct(MethodRef {
        class: extends.clone(),
        name: CONSTRUCTOR.to_owned(),
        descriptor: MethodDescriptor::new(tag_taken(shape), None),
    }));
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
    instructions.push(Instruction::Return(None));
    instance_method(CONSTRUCTOR, shape.constructor(), instructions, 0)
}

/// The base's own constructor, which takes the tag and is all a base does.
fn base_constructor(base: &ClassName) -> Method {
    let this = Descriptor::Reference(base.clone());
    let instructions = vec![
        Instruction::Load {
            slot: 0,
            of: this.clone(),
        },
        Instruction::Construct(MethodRef {
            class: object_class(),
            name: CONSTRUCTOR.to_owned(),
            descriptor: MethodDescriptor::new(Vec::new(), None),
        }),
        Instruction::Load { slot: 0, of: this },
        Instruction::Load {
            slot: 1,
            of: Descriptor::Integer,
        },
        Instruction::PutField(FieldRef {
            class: base.clone(),
            name: TAG.to_owned(),
            of: Descriptor::Integer,
        }),
        Instruction::Return(None),
    ];
    let descriptor = MethodDescriptor::new(vec![Descriptor::Integer], None);
    instance_method(CONSTRUCTOR, descriptor, instructions, 0)
}

/// Two values of the class are the same when they hold the same values, field by field.
fn equals(shape: &Shape) -> Method {
    let descriptor = MethodDescriptor::new(vec![object()], Some(Descriptor::Boolean));
    instance_method(EQUALS, descriptor, equals_body(shape), 1)
}

/// The body of `equals`: the other value is of this class, and holds what this one holds.
fn equals_body(shape: &Shape) -> Vec<Instruction> {
    let mut instructions = vec![
        Instruction::Load {
            slot: 1,
            of: object(),
        },
        Instruction::InstanceOf(shape.class.clone()),
        Instruction::JumpIfFalse(DIFFERENT),
        Instruction::Load {
            slot: 1,
            of: object(),
        },
        Instruction::Cast(shape.class.clone()),
        Instruction::Store {
            slot: 2,
            of: Descriptor::Reference(shape.class.clone()),
        },
    ];
    for field in held(shape) {
        instructions.extend(field_compared(shape, &field));
    }
    instructions.extend([
        Instruction::Boolean(true),
        Instruction::Return(Some(Descriptor::Boolean)),
        Instruction::Label(DIFFERENT),
        Instruction::Boolean(false),
        Instruction::Return(Some(Descriptor::Boolean)),
    ]);
    instructions
}

/// One field of both values, compared, leaving the comparison to go on when they agree.
fn field_compared(shape: &Shape, field: &Field) -> Vec<Instruction> {
    let this = Descriptor::Reference(shape.class.clone());
    let mut instructions = vec![
        Instruction::Load {
            slot: 0,
            of: this.clone(),
        },
        Instruction::GetField(reached(shape, field)),
        Instruction::Load { slot: 2, of: this },
        Instruction::GetField(reached(shape, field)),
    ];
    instructions.extend(compared(Some(&field.of), Comparison::Equal));
    instructions.push(Instruction::JumpIfFalse(DIFFERENT));
    instructions
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

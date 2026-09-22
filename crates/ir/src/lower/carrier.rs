//! What carries a list: a buffer shared between lists, and the length of one of them.
//!
//! `docs/specs/codegen.md` states it. The class is written with the prelude types on every
//! build, and it declares its constructor and three static methods: `of` builds a list from an
//! array, `push` grows one, and `listed` gives the `java.util.List` a Java member takes.
//!
//! A slot of the buffer is filled when it holds an element, and no element is `null`, so the
//! first slot that holds `null` is the first one no list of that buffer holds yet.

use crate::class::{Class, Method, Reached};
use crate::code::{Arithmetic, Body, Comparison, FieldRef, Instruction, Label, MethodRef};
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
use crate::lower::classes::record_class;
use crate::lower::shape::{CONSTRUCTOR, Carried, Shape, object, object_class};

/// The class that carries every list, which is in the package of the prelude types.
pub(crate) const LIST: &str = "lumen/List";

/// The field that holds the buffer, which lists of one buffer share.
const SLOTS: &str = "slots";

/// The field that holds how many slots of the buffer this list holds.
const LENGTH: &str = "length";

/// The method that builds a list from a full array.
const OF: &str = "of";

/// The method that grows a list by one element.
const PUSH: &str = "push";

/// The method that gives the `java.util.List` a Java member takes.
const LISTED: &str = "listed";

/// What a Java member takes a list as.
const JAVA_LIST: &str = "java/util/List";

/// The list, which is the first parameter of each method.
const THE_LIST: u16 = 0;

/// The value a push appends, and the array a crossing copies into.
const SECOND: u16 = 1;

/// The buffer a push writes into, once it is read out of the list.
const BUFFER: u16 = 2;

/// The length of the list a push was handed, which is the index of the slot it fills.
const AT: u16 = 3;

/// The new buffer a push copies into, where it copies.
const FRESH: u16 = 4;

/// Where a push goes when the next slot is not free.
const COPY: Label = Label(0);

/// Where a push writes the element, into a slot that is now free.
const CLAIM: Label = Label(1);

/// The count a crossing keeps as it walks the list.
const INDEX: u16 = 2;

/// The element a crossing reads, and then what it puts in the array in its place.
const ELEMENT: u16 = 3;

/// Where each turn of a crossing begins.
const WALK: Label = Label(0);

/// Where a crossing puts the element into the array, once it is what a member takes.
const PUT: Label = Label(1);

/// Where a crossing ends, after it put each element.
const WALKED: Label = Label(2);

/// The class that carries a list, with its constructor and its three static methods.
pub(crate) fn carrier() -> Class {
    let mut class = record_class(&shape());
    class.methods.extend([of(), push(), listed()]);
    class
}

/// `of(array)`: a list that holds every slot of the array, which is full.
fn of() -> Method {
    let instructions = vec![
        Instruction::New(ClassName::new(LIST)),
        Instruction::Copy,
        loaded(THE_LIST, buffer()),
        loaded(THE_LIST, buffer()),
        Instruction::ArrayLength,
        constructed(),
        Instruction::Return(Some(list())),
    ];
    static_method(built(), instructions, 0)
}

/// `push(list, value)`: claims the next slot where it is free, and copies first where it is not.
///
/// The next slot is free when the buffer has one past the list, and no push has filled it yet.
fn push() -> Method {
    let mut instructions = vec![
        loaded(THE_LIST, list()),
        Instruction::GetField(slots()),
        stored(BUFFER, buffer()),
        loaded(THE_LIST, list()),
        Instruction::GetField(length()),
        stored(AT, Descriptor::Integer),
        loaded(AT, Descriptor::Integer),
        loaded(BUFFER, buffer()),
        Instruction::ArrayLength,
        Instruction::CompareIntegers(Comparison::Less),
        Instruction::JumpIfFalse(COPY),
        loaded(BUFFER, buffer()),
        loaded(AT, Descriptor::Integer),
        Instruction::LoadFromArray,
        Instruction::JumpIfNull(CLAIM),
        Instruction::Label(COPY),
    ];
    instructions.extend(copied());
    instructions.extend([
        Instruction::Label(CLAIM),
        loaded(BUFFER, buffer()),
        loaded(AT, Descriptor::Integer),
        loaded(SECOND, object()),
        Instruction::StoreInArray,
        Instruction::Increment { slot: AT },
        Instruction::New(ClassName::new(LIST)),
        Instruction::Copy,
        loaded(BUFFER, buffer()),
        loaded(AT, Descriptor::Integer),
        constructed(),
        Instruction::Return(Some(list())),
    ]);
    static_method(pushed(), instructions, 3)
}

/// The first `length` elements copied into a new buffer, which is the buffer from here on.
///
/// The new buffer has twice the length and one slot more, so an empty list grows too. A list
/// holds fewer elements than a JVM array has slots, and a JVM holds no array near that length,
/// so twice the length and one more is a small whole number.
fn copied() -> Vec<Instruction> {
    vec![
        loaded(AT, Descriptor::Integer),
        Instruction::Widen,
        Instruction::Long(2),
        Instruction::Arithmetic(Arithmetic::Multiply),
        Instruction::Long(1),
        Instruction::Arithmetic(Arithmetic::Add),
        Instruction::Narrow,
        Instruction::NewArray(object_class()),
        stored(FRESH, buffer()),
        loaded(BUFFER, buffer()),
        Instruction::Integer(0),
        loaded(FRESH, buffer()),
        Instruction::Integer(0),
        loaded(AT, Descriptor::Integer),
        Instruction::InvokeStatic(array_copy()),
        loaded(FRESH, buffer()),
        stored(BUFFER, buffer()),
    ]
}

/// `System.arraycopy`, which copies the first elements of one array into another.
fn array_copy() -> MethodRef {
    MethodRef {
        class: ClassName::new("java/lang/System"),
        name: "arraycopy".to_owned(),
        descriptor: MethodDescriptor::new(
            vec![
                object(),
                Descriptor::Integer,
                object(),
                Descriptor::Integer,
                Descriptor::Integer,
            ],
            None,
        ),
    }
}

/// `listed(list)`: the `java.util.List` of the first `length` elements, in an array of its own.
///
/// The member reaches that array and nothing else, so it cannot change what the list holds. An
/// element that is a list itself crosses the same way, so a member that takes a list of lists
/// finds a `java.util.List` at each element.
fn listed() -> Method {
    let counted = [loaded(THE_LIST, list()), Instruction::GetField(length())];
    let mut instructions = counted.to_vec();
    instructions.extend([
        Instruction::NewArray(object_class()),
        stored(SECOND, buffer()),
        Instruction::Integer(0),
        stored(INDEX, Descriptor::Integer),
        Instruction::Label(WALK),
        loaded(INDEX, Descriptor::Integer),
    ]);
    instructions.extend(counted);
    instructions.extend([
        Instruction::CompareIntegers(Comparison::Less),
        Instruction::JumpIfFalse(WALKED),
        loaded(THE_LIST, list()),
        Instruction::GetField(slots()),
        loaded(INDEX, Descriptor::Integer),
        Instruction::LoadFromArray,
        stored(ELEMENT, object()),
        loaded(ELEMENT, object()),
        Instruction::InstanceOf(ClassName::new(LIST)),
        Instruction::JumpIfFalse(PUT),
        loaded(ELEMENT, object()),
        Instruction::Cast(ClassName::new(LIST)),
        Instruction::InvokeStatic(crossed()),
        stored(ELEMENT, object()),
        Instruction::Label(PUT),
        loaded(SECOND, buffer()),
        loaded(INDEX, Descriptor::Integer),
        loaded(ELEMENT, object()),
        Instruction::StoreInArray,
        Instruction::Increment { slot: INDEX },
        Instruction::Jump(WALK),
        Instruction::Label(WALKED),
        loaded(SECOND, buffer()),
        Instruction::InvokeStaticOfInterface(MethodRef {
            class: ClassName::new(JAVA_LIST),
            name: OF.to_owned(),
            descriptor: MethodDescriptor::new(vec![buffer()], Some(java_list())),
        }),
        Instruction::Return(Some(java_list())),
    ]);
    static_method(crossed(), instructions, 3)
}

/// The constructor, which takes the buffer and the length.
fn constructed() -> Instruction {
    Instruction::Construct(MethodRef {
        class: ClassName::new(LIST),
        name: CONSTRUCTOR.to_owned(),
        descriptor: shape().constructor(),
    })
}

/// What the class holds, laid out as a record's fields are.
fn shape() -> Shape {
    let class = ClassName::new(LIST);
    Shape {
        class: class.clone(),
        base: class,
        tag: None,
        carries: vec![
            Carried {
                name: SLOTS.to_owned(),
                of: Some(buffer()),
            },
            Carried {
                name: LENGTH.to_owned(),
                of: Some(Descriptor::Integer),
            },
        ],
    }
}

/// The call that builds a list from the array on the stack.
pub(crate) fn built() -> MethodRef {
    method(OF, vec![buffer()], list())
}

/// The call that pushes the value on the stack onto the list below it.
pub(crate) fn pushed() -> MethodRef {
    method(PUSH, vec![list(), object()], list())
}

/// The call that gives the `java.util.List` of what the list on the stack holds.
pub(crate) fn crossed() -> MethodRef {
    method(LISTED, vec![list()], java_list())
}

/// What a Java member takes a list as, which `listed` gives.
pub(crate) fn java_list() -> Descriptor {
    Descriptor::reference(JAVA_LIST)
}

/// The field that holds the buffer of a list.
pub(crate) fn slots() -> FieldRef {
    field(SLOTS, buffer())
}

/// The field that holds how many slots of the buffer a list holds.
pub(crate) fn length() -> FieldRef {
    field(LENGTH, Descriptor::Integer)
}

/// A reference to a list, which is what a value of `List<T>` is carried by.
pub(crate) fn list() -> Descriptor {
    Descriptor::reference(LIST)
}

/// What a buffer is, which is an array of references.
fn buffer() -> Descriptor {
    Descriptor::array(object())
}

fn loaded(slot: u16, of: Descriptor) -> Instruction {
    Instruction::Load { slot, of }
}

fn stored(slot: u16, of: Descriptor) -> Instruction {
    Instruction::Store { slot, of }
}

fn field(name: &str, of: Descriptor) -> FieldRef {
    FieldRef {
        class: ClassName::new(LIST),
        name: name.to_owned(),
        of,
    }
}

fn method(name: &str, parameters: Vec<Descriptor>, result: Descriptor) -> MethodRef {
    MethodRef {
        class: ClassName::new(LIST),
        name: name.to_owned(),
        descriptor: MethodDescriptor::new(parameters, Some(result)),
    }
}

/// A static method of the class, with `locals` slots beyond the ones its parameters arrive in.
fn static_method(method: MethodRef, instructions: Vec<Instruction>, locals: u16) -> Method {
    Method {
        name: method.name,
        descriptor: method.descriptor,
        reached: Reached::ThroughTheClass,
        body: Body {
            instructions,
            locals,
            guards: Vec::new(),
        },
    }
}

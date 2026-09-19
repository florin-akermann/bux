//! What a method body does, as a list of instructions over a stack.

use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};

/// A place a jump can land, named by a number the body hands out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Label(pub u32);

/// One field of one class, which is what reading or writing a field names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldRef {
    pub class: ClassName,
    pub name: String,
    pub of: Descriptor,
}

/// One method of one class, which is what a call names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodRef {
    pub class: ClassName,
    pub name: String,
    pub descriptor: MethodDescriptor,
}

/// How two values of one type are being compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

/// What is being done to two whole numbers, or to one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arithmetic {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Negate,
}

/// One step of a method body.
///
/// The set is the one version 0.1 lowers to and no more: an instruction is here because some
/// construct of the language becomes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    /// Pushes a whole number.
    Long(i64),
    /// Pushes a truth value.
    Boolean(bool),
    /// Pushes a small whole number, which is how a tag is written.
    Integer(i32),
    /// Pushes a string.
    Text(String),
    /// Pushes what a local holds.
    Load {
        slot: u16,
        of: Descriptor,
    },
    /// Takes the top of the stack into a local.
    Store {
        slot: u16,
        of: Descriptor,
    },
    /// Drops the top of the stack, which is how a value evaluated for its effect ends.
    Drop(Descriptor),
    /// Copies the top of the stack, which is a reference wherever version 0.1 does it.
    Copy,
    Arithmetic(Arithmetic),
    /// Joins two strings.
    Concat,
    /// Compares two whole numbers, leaving a truth value.
    CompareLongs(Comparison),
    /// Compares two small whole numbers or truth values, leaving a truth value.
    CompareIntegers(Comparison),
    /// Turns a truth value into the other one.
    Not,
    /// Names a place a jump lands.
    Label(Label),
    Jump(Label),
    /// Jumps when the truth value on the stack is false.
    JumpIfFalse(Label),
    /// Makes an uninitialised instance, which a constructor then takes.
    New(ClassName),
    /// Runs a constructor over the values above the instance it initialises.
    Construct(MethodRef),
    /// Reads a field of the instance on the stack.
    GetField(FieldRef),
    /// Writes the value on the stack into a field of the instance below it.
    PutField(FieldRef),
    InvokeStatic(MethodRef),
    InvokeVirtual(MethodRef),
    /// Calls a method of an interface, which is how a `for … in` walks a list.
    InvokeInterface(MethodRef),
    /// Refuses the value on the stack unless it is an instance of the class.
    Cast(ClassName),
    /// Asks whether the value on the stack is an instance of the class, leaving a truth value.
    InstanceOf(ClassName),
    /// Adds one to the small whole number a local holds, which is the count a `for … in` keeps.
    Increment {
        slot: u16,
    },
    /// Leaves the method, giving back the top of the stack when there is one.
    Return(Option<Descriptor>),
    /// Leaves the method by throwing what is on the stack, which nothing catches.
    Throw,
}

/// A method body: what it does, and the locals it does it with.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Body {
    pub instructions: Vec<Instruction>,
    /// The slots the body uses beyond the ones its parameters arrive in.
    pub locals: u16,
}

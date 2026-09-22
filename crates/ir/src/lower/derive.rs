//! The body of a method a derive asks for, which is the one an author would have written.
//!
//! `docs/specs/derive.md` states what each of the four reads: a record field by field in
//! declaration order, and a variant by which variant it is and then by what that variant carries.
//! Each value is handed to the instance of its own type, so this reads the declaration and calls
//! what that type already has, exactly as a written instance would.

mod equality;
mod hash;
mod order;
mod show;

use lumen_ast::{RecordField, TypeDeclaration, TypeDefinition, TypeRef, TypeRefKind};
use lumen_ast::{Variant, VariantPayload};
use lumen_resolver::prelude;
use lumen_types::Type;

use crate::code::{Body, Comparison, FieldRef, Instruction, Label};
use crate::descriptor::{ClassName, Descriptor};
use crate::lower::Lowering;
use crate::lower::shape::{Carried, Shape, TAG};
use crate::lower::standard::{compared, hashed_as, shown_as};

/// The local the first of the values a derived method takes arrives in.
const ONE: u16 = 0;

/// The local the second of them arrives in, which `Hash` and `Show` never have.
const OTHER: u16 = 1;

/// The body of the method `derive of for T` writes, read off the declaration of `T`.
pub(crate) fn body(lowering: &Lowering<'_>, of: &str, declared: &TypeDeclaration) -> Body {
    let mut writing = Writing {
        lowering,
        instructions: Vec::new(),
        next_label: 0,
    };
    match of {
        prelude::EQ => equality::is_equal(&mut writing, declared),
        prelude::ORD => order::is_less(&mut writing, declared),
        prelude::HASH => hash::hashed(&mut writing, declared),
        prelude::SHOW => show::shown(&mut writing, declared),
        _ => unreachable!("name resolution refused a derive of anything but a standard trait"),
    }
    Body {
        instructions: writing.instructions,
        locals: 0,
        guards: Vec::new(),
    }
}

/// One value a derived body reads: where it lives in the class, what it is called in the source,
/// and the type it was written as.
struct Holds<'a> {
    shape: Shape,
    carried: Carried,
    of: Descriptor,
    /// The field's name, where the source gave what it holds one.
    named: Option<&'a str>,
    written: &'a TypeRef,
}

/// One derived body part-way through being written.
struct Writing<'a> {
    lowering: &'a Lowering<'a>,
    instructions: Vec<Instruction>,
    next_label: u32,
}

impl<'a> Writing<'a> {
    /// Every value a record holds, in the order the type declares its fields.
    fn record_holds(&self, named: &str, fields: &'a [RecordField]) -> Vec<Holds<'a>> {
        let shape = self.lowering.shapes.record(named).clone();
        let named = fields.iter().map(|field| Some(field.name.text.as_str()));
        let written = fields.iter().map(|field| &field.type_ref);
        held(&shape, named.zip(written))
    }

    /// Every value one variant carries, in the order it carries them.
    fn variant_holds(&self, variant: &'a Variant) -> Vec<Holds<'a>> {
        let shape = self.shape_of(variant);
        match &variant.payload {
            VariantPayload::None => Vec::new(),
            VariantPayload::Tuple(written) => held(&shape, written.iter().map(|one| (None, one))),
            VariantPayload::Record(fields) => {
                let named = fields.iter().map(|field| Some(field.name.text.as_str()));
                let written = fields.iter().map(|field| &field.type_ref);
                held(&shape, named.zip(written))
            }
        }
    }

    /// The class one variant builds, which is what its tag names and what its values live in.
    fn shape_of(&self, variant: &Variant) -> Shape {
        self.lowering.shapes.built(&variant.name.text).clone()
    }

    /// Reads one value off what a local holds, which a variant is cast to first.
    fn read(&mut self, holds: &Holds<'_>, slot: u16) {
        let shape = &holds.shape;
        self.emit(Instruction::Load {
            slot,
            of: Descriptor::Reference(shape.base.clone()),
        });
        if shape.class != shape.base {
            self.emit(Instruction::Cast(shape.class.clone()));
        }
        self.emit(Instruction::GetField(FieldRef {
            class: shape.class.clone(),
            name: holds.carried.name.clone(),
            of: holds.of.clone(),
        }));
    }

    /// Calls the instance of `of` at the type one value was written as, over what is on the stack.
    ///
    /// A type whose instance a module wrote or derived is an `invokestatic` of that instance's
    /// method; one of the types the JVM holds is what that instance amounts to, written out.
    fn through(&mut self, of: &str, holds: &Holds<'_>) {
        let at = written_as(holds.written);
        let method = prelude::method_of(of).expect("a derivable trait declares one method");
        let Some(instance) = self.lowering.answering(method, &at) else {
            for instruction in supplied(of, &holds.of) {
                self.emit(instruction);
            }
            return;
        };
        self.emit(instance.called());
    }

    /// Jumps to `elsewhere` unless what `ONE` holds is the variant `shape` builds.
    fn unless_it_is(&mut self, shape: &Shape, elsewhere: Label) {
        let Some(tag) = shape.tag else {
            unreachable!("only a variant is asked which one it is, and every variant has a tag")
        };
        self.tag(&shape.base, ONE);
        self.emit(Instruction::Integer(tag));
        self.emit(Instruction::CompareIntegers(Comparison::Equal));
        self.emit(Instruction::JumpIfFalse(elsewhere));
    }

    /// The tag of what one local holds, which says which variant that value is.
    fn tag(&mut self, base: &ClassName, slot: u16) {
        self.emit(Instruction::Load {
            slot,
            of: Descriptor::Reference(base.clone()),
        });
        self.emit(Instruction::GetField(FieldRef {
            class: base.clone(),
            name: TAG.to_owned(),
            of: Descriptor::Integer,
        }));
    }

    /// The base class of a type declaring variants, which both locals hold a value as.
    fn base_of(&self, variants: &[Variant]) -> ClassName {
        let Some(first) = variants.first() else {
            unreachable!("a type declaring variants declares at least one")
        };
        self.lowering.shapes.built(&first.name.text).base.clone()
    }

    fn label(&mut self) -> Label {
        let label = Label(self.next_label);
        self.next_label += 1;
        label
    }

    fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
}

/// The type one value was written as, with every argument it is written with.
///
/// The arguments are read as well as the name: a field written `List<Int>` reaches the instance
/// written over `List<T>` at `Int`, which is not the one `List<Bool>` reaches.
fn written_as(written: &TypeRef) -> Type {
    let TypeRefKind::Named { path, arguments } = &written.kind else {
        unreachable!("`()` has no instance of anything, so nothing holding one derives")
    };
    Type::Named {
        name: path.to_string(),
        arguments: arguments.iter().map(written_as).collect(),
    }
}

/// What the prelude's own instance of `of` amounts to, over a value the JVM holds as `held`.
fn supplied(of: &str, held: &Descriptor) -> Vec<Instruction> {
    match of {
        prelude::EQ => compared(Some(held), Comparison::Equal),
        prelude::ORD => compared(Some(held), Comparison::Less),
        prelude::HASH => hashed_as(held),
        prelude::SHOW => shown_as(held),
        _ => unreachable!("name resolution refused a derive of anything but a standard trait"),
    }
}

/// The values `shape` holds, paired with what the source called each and wrote it as.
///
/// A value carried by nothing is left out: `docs/specs/derive.md` refuses a `()` before this is
/// reached, so nothing a derived body reads is ever a value there is only one of.
fn held<'a>(
    shape: &Shape,
    written: impl Iterator<Item = (Option<&'a str>, &'a TypeRef)>,
) -> Vec<Holds<'a>> {
    shape
        .carries
        .iter()
        .zip(written)
        .filter_map(|(carried, (named, written))| {
            Some(Holds {
                shape: shape.clone(),
                carried: carried.clone(),
                of: carried.of.clone()?,
                named,
                written,
            })
        })
        .collect()
}

/// Every value a type holds, where it holds them all in one block: a record's fields.
fn record_of(declared: &TypeDeclaration) -> Option<&[RecordField]> {
    match &declared.definition {
        TypeDefinition::Record(fields) => Some(fields),
        TypeDefinition::Variants(_) | TypeDefinition::Foreign { .. } => None,
    }
}

/// The variants a type declares, where it declares variants rather than fields.
fn variants_of(declared: &TypeDeclaration) -> Option<&[Variant]> {
    match &declared.definition {
        TypeDefinition::Record(_) | TypeDefinition::Foreign { .. } => None,
        TypeDefinition::Variants(variants) => Some(variants),
    }
}

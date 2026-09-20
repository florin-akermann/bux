//! The body of a method a derive asks for, which is the one an author would have written.
//!
//! `docs/specs/derive.md` states what a derived `Eq` compares: a record field by field in
//! declaration order, and a variant by its tag and then by what that variant carries. Each value
//! is compared by the `Eq` of its own type, so this reads the declaration and calls what the
//! field's type already has, exactly as a written instance would.

use lumen_ast::{RecordField, TypeDeclaration, TypeDefinition, TypeRef, TypeRefKind};
use lumen_ast::{Variant, VariantPayload};
use lumen_resolver::prelude;
use lumen_types::Type;

use crate::code::{Body, Comparison, FieldRef, Instruction, Label, MethodRef};
use crate::descriptor::{ClassName, Descriptor};
use crate::lower::Lowering;
use crate::lower::equality::compared;
use crate::lower::shape::{Carried, Shape, TAG};

/// The local the first of the two values arrives in.
const ONE: u16 = 0;

/// The local the second of the two values arrives in.
const OTHER: u16 = 1;

/// The body of `Eq$T$is_equal` for the type `declared`, which compares two values by their state.
pub(crate) fn is_equal(lowering: &Lowering<'_>, declared: &TypeDeclaration) -> Body {
    let mut writing = Writing {
        lowering,
        instructions: Vec::new(),
        next_label: 0,
    };
    let differs = writing.label();
    match &declared.definition {
        TypeDefinition::Record(fields) => writing.record(&declared.name.text, fields, differs),
        TypeDefinition::Variants(variants) => writing.variants(variants, differs),
    }
    writing.answer(differs);
    Body {
        instructions: writing.instructions,
        locals: 0,
        guards: Vec::new(),
    }
}

/// One value both sides hold: where it lives in the class, and the type it was written as.
struct Holds<'a> {
    shape: &'a Shape,
    carried: &'a Carried,
    written: &'a TypeRef,
}

/// One derived body part-way through being written.
struct Writing<'a> {
    lowering: &'a Lowering<'a>,
    instructions: Vec<Instruction>,
    next_label: u32,
}

impl Writing<'_> {
    /// A record: each field compared in the order the type declares them.
    fn record(&mut self, named: &str, fields: &[RecordField], differs: Label) {
        let shape = self.lowering.shapes.record(named).clone();
        for (carried, field) in shape.carries.iter().zip(fields) {
            let holds = Holds {
                shape: &shape,
                carried,
                written: &field.type_ref,
            };
            self.compared(&holds, differs);
        }
    }

    /// Variants: the same variant first, and then whatever that variant carries.
    ///
    /// The tags settle which variant both values are, so the walk that follows reaches the one
    /// block that carries anything and jumps over every other.
    fn variants(&mut self, variants: &[Variant], differs: Label) {
        let equal = self.label();
        let Some(first) = variants.first() else {
            unreachable!("a type declaring variants declares at least one")
        };
        let base = self.lowering.shapes.built(&first.name.text).base.clone();
        self.tag(&base, ONE);
        self.tag(&base, OTHER);
        self.emit(Instruction::CompareIntegers(Comparison::Equal));
        self.emit(Instruction::JumpIfFalse(differs));
        for variant in variants {
            self.payload(variant, equal, differs);
        }
        self.emit(Instruction::Label(equal));
    }

    /// What one variant carries, compared where both values turn out to be that variant.
    fn payload(&mut self, variant: &Variant, equal: Label, differs: Label) {
        let shape = self.lowering.shapes.built(&variant.name.text).clone();
        let held = carried_by(variant);
        if held.is_empty() {
            return;
        }
        let next = self.label();
        self.tag(&shape.base, ONE);
        self.emit(Instruction::Integer(shape.tag.unwrap_or_default()));
        self.emit(Instruction::CompareIntegers(Comparison::Equal));
        self.emit(Instruction::JumpIfFalse(next));
        for (carried, written) in shape.carries.iter().zip(held) {
            let holds = Holds {
                shape: &shape,
                carried,
                written,
            };
            self.compared(&holds, differs);
        }
        self.emit(Instruction::Jump(equal));
        self.emit(Instruction::Label(next));
    }

    /// One value both sides hold, compared by the `Eq` of its own type.
    fn compared(&mut self, holds: &Holds<'_>, differs: Label) {
        let Some(of) = holds.carried.of.clone() else {
            return;
        };
        self.read(holds, &of, ONE);
        self.read(holds, &of, OTHER);
        self.same(holds.written, &of);
        self.emit(Instruction::JumpIfFalse(differs));
    }

    /// Reads one value off what the local holds, which a variant is cast to first.
    fn read(&mut self, holds: &Holds<'_>, of: &Descriptor, slot: u16) {
        let shape = holds.shape;
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
            of: of.clone(),
        }));
    }

    /// Whether the two values above it on the stack are the same, by the `Eq` of their type.
    ///
    /// A type whose instance a module wrote or derived is an `invokestatic` of that instance's
    /// method; a type the compiler supplies the instance for is the comparison it always was.
    fn same(&mut self, written: &TypeRef, of: &Descriptor) {
        let TypeRefKind::Named { name, .. } = &written.kind else {
            unreachable!("`()` has no `Eq`, so nothing holding one derives `Eq`")
        };
        let at = Type::Named {
            name: name.text.clone(),
            arguments: Vec::new(),
        };
        let Some(declared) = self.lowering.answering(prelude::IS_EQUAL, &at) else {
            for instruction in compared(Some(of), Comparison::Equal) {
                self.emit(instruction);
            }
            return;
        };
        let reached = self.lowering.plainly(declared);
        self.emit(Instruction::InvokeStatic(MethodRef {
            class: self.lowering.shapes.module().clone(),
            name: reached.named,
            descriptor: reached.signature.descriptor(),
        }));
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

    /// The two answers and the one way out, which every comparison above has jumped into.
    fn answer(&mut self, differs: Label) {
        let end = self.label();
        self.emit(Instruction::Boolean(true));
        self.emit(Instruction::Jump(end));
        self.emit(Instruction::Label(differs));
        self.emit(Instruction::Boolean(false));
        self.emit(Instruction::Label(end));
        self.emit(Instruction::Return(Some(Descriptor::Boolean)));
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

/// Every value one variant carries, as the types they are written with.
fn carried_by(variant: &Variant) -> Vec<&TypeRef> {
    match &variant.payload {
        VariantPayload::None => Vec::new(),
        VariantPayload::Tuple(written) => written.iter().collect(),
        VariantPayload::Record(fields) => fields.iter().map(|field| &field.type_ref).collect(),
    }
}

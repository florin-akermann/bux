//! The body of `Show$T$shown`, which renders a value as the source that builds it.
//!
//! `docs/specs/derive.md` writes a record as its type's name and its fields in declaration order,
//! and a variant as its own name and whatever it carries. Each value is rendered by the `Show` of
//! its own type, so a `String` field shows its characters without quotes.

use lumen_ast::{TypeDeclaration, Variant, VariantPayload};
use lumen_resolver::prelude;

use crate::code::Instruction;
use crate::descriptor::Descriptor;
use crate::lower::derive::{Holds, ONE, Writing, record_of, variants_of};

/// Writes the body: the text the value is shown as, and the answer.
pub(super) fn shown(writing: &mut Writing<'_>, declared: &TypeDeclaration) {
    if let Some(fields) = record_of(declared) {
        let held = writing.record_holds(&declared.name.text, fields);
        inside_braces(writing, &declared.name.text, &held);
    }
    if let Some(variants) = variants_of(declared) {
        by_variant(writing, variants);
    }
    writing.emit(Instruction::Return(Some(Descriptor::reference(
        "java/lang/String",
    ))));
}

/// Variants: the one the value turns out to be, rendered as its own name and what it carries.
///
/// The last variant is written without a test, because the tags are every variant there is and
/// the one the value is has to be among them.
fn by_variant(writing: &mut Writing<'_>, variants: &[Variant]) {
    let end = writing.label();
    let Some((last, rest)) = variants.split_last() else {
        unreachable!("a type declaring variants declares at least one")
    };
    for variant in rest {
        let next = writing.label();
        let shape = writing.shape_of(variant);
        writing.unless_it_is(&shape, next);
        one_variant(writing, variant);
        writing.emit(Instruction::Jump(end));
        writing.emit(Instruction::Label(next));
    }
    one_variant(writing, last);
    writing.emit(Instruction::Label(end));
}

/// One variant: its name alone, its name and what it carries in order, or its name and its fields.
///
/// Which of the three it is follows from how the variant is declared and not from how much it
/// carries, so a variant declaring braces and no field inside them is shown with those braces.
fn one_variant(writing: &mut Writing<'_>, variant: &Variant) {
    let named = &variant.name.text;
    let held = writing.variant_holds(variant);
    match &variant.payload {
        VariantPayload::None => writing.emit(Instruction::Text(named.clone())),
        VariantPayload::Tuple(_) => inside_brackets(writing, named, &held),
        VariantPayload::Record(_) => inside_braces(writing, named, &held),
    }
}

/// `User { id: 1, name: ada }`: the name, then each field as the source names it.
fn inside_braces(writing: &mut Writing<'_>, named: &str, held: &[Holds<'_>]) {
    writing.emit(Instruction::Text(format!("{named} {{")));
    for (position, holds) in held.iter().enumerate() {
        let opening = if position == 0 { " " } else { ", " };
        let Some(field) = holds.named else {
            unreachable!("what a record and a record payload hold is written with a name each")
        };
        joined(writing, Instruction::Text(format!("{opening}{field}: ")));
        showing(writing, holds);
    }
    let closing = if held.is_empty() { "}" } else { " }" };
    joined(writing, Instruction::Text(closing.to_owned()));
}

/// `Failed(late)`: the name, then each value it carries in the order it carries them.
fn inside_brackets(writing: &mut Writing<'_>, named: &str, held: &[Holds<'_>]) {
    writing.emit(Instruction::Text(format!("{named}(")));
    for (position, holds) in held.iter().enumerate() {
        if position > 0 {
            joined(writing, Instruction::Text(", ".to_owned()));
        }
        showing(writing, holds);
    }
    joined(writing, Instruction::Text(")".to_owned()));
}

/// One value the type holds, shown by the `Show` of its own type and joined to what is below it.
fn showing(writing: &mut Writing<'_>, holds: &Holds<'_>) {
    writing.read(holds, ONE);
    writing.through(prelude::SHOW, holds);
    writing.emit(Instruction::Concat);
}

/// Puts more text on the end of the text already on the stack.
fn joined(writing: &mut Writing<'_>, more: Instruction) {
    writing.emit(more);
    writing.emit(Instruction::Concat);
}

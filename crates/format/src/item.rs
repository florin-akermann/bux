//! The top-level declarations of a source file.

use lumen_ast::{DeriveDeclaration, ExternDeclaration, Function, Import, InstanceDeclaration};
use lumen_ast::{Item, Name};
use lumen_ast::{Parameter, Program};
use lumen_ast::{RecordField, Signature, Span, TraitDeclaration, TypeDeclaration};
use lumen_ast::{TypeDefinition, TypeParameter, TypeRef, Variant, VariantPayload};

use crate::literal::string;
use crate::printer::Printer;
use crate::stmt::{block, close_block, open_block};
use crate::type_ref::type_ref;

/// Every item of the file, with exactly one blank line between two of them.
pub(crate) fn program(printer: &mut Printer, written: &Program, end: usize) {
    for (position, declared) in written.items.iter().enumerate() {
        if position > 0 {
            printer.blank_line();
        }
        item(printer, declared);
    }
    trailing_comments(printer, !written.items.is_empty(), end);
}

fn item(printer: &mut Printer, written: &Item) {
    match written {
        Item::Import(imported) => import(printer, imported),
        Item::Type(declared) => type_declaration(printer, declared),
        Item::Trait(declared) => trait_declaration(printer, declared),
        Item::Instance(declared) => instance(printer, declared),
        Item::Derive(declared) => derive(printer, declared),
        Item::Function(declared) => function(printer, declared),
        Item::Extern(declared) => declared_extern(printer, declared),
    }
}

/// `extern static read(path: Path) -> String = "java.nio.file.Files.readString"`, on one line.
///
/// There is no body to open a block with, and the result is always written, so every one of
/// these is the one line `docs/specs/interop.md` writes it as.
pub(crate) fn declared_extern(printer: &mut Printer, written: &ExternDeclaration) {
    printer.comments_above(written.span);
    printer.open_line();
    printer.word("extern ");
    printer.word(written.reaches.written());
    printer.word(" ");
    printer.word(&written.name.text);
    parameters(printer, &written.parameters);
    printer.word(" -> ");
    type_ref(printer, &written.result);
    if let Some(named) = written.reaches.named() {
        printer.word(" = ");
        printer.word(&string(&named.text));
    }
    printer.end_line();
}

/// A comment after the last item, at the left margin and below one blank line.
fn trailing_comments(printer: &mut Printer, after_an_item: bool, end: usize) {
    if !printer.has_comments() {
        return;
    }
    if after_an_item {
        printer.blank_line();
    }
    printer.comments_before(end);
}

fn import(printer: &mut Printer, written: &Import) {
    printer.comments_above(written.span);
    printer.open_line();
    printer.word("import ");
    printer.word(&written.module.text);
    printer.end_line();
}

pub(crate) fn type_declaration(printer: &mut Printer, written: &TypeDeclaration) {
    printer.comments_above(written.span);
    printer.open_line();
    match &written.definition {
        TypeDefinition::Foreign(class) => {
            printer.word("extern type ");
            printer.word(&written.name.text);
            printer.word(" = ");
            printer.word(&string(&class.text));
            printer.end_line();
        }
        TypeDefinition::Record(fields) => {
            opened(printer, written);
            record_body(printer, fields, written.span);
            printer.end_line();
        }
        TypeDefinition::Variants(variants) => {
            opened(printer, written);
            variants_of(printer, variants, written.span);
        }
    }
}

/// `type User<T> =`, which every type a declaration writes the body of opens with.
fn opened(printer: &mut Printer, written: &TypeDeclaration) {
    printer.word("type ");
    printer.word(&written.name.text);
    type_parameters(printer, &written.parameters);
    printer.word(" =");
}

/// One variant on the `type` line, or two or more with a `|` each, one per line.
fn variants_of(printer: &mut Printer, variants: &[Variant], enclosing: Span) {
    if let [only] = variants {
        printer.word(" ");
        variant(printer, only, enclosing);
        printer.end_line();
        return;
    }
    printer.end_line();
    printer.indent();
    for declared in variants {
        printer.comments_above(declared.span);
        printer.open_line();
        printer.word("| ");
        variant(printer, declared, declared.span);
        printer.end_line();
    }
    printer.comments_before(enclosing.end());
    printer.dedent();
}

fn variant(printer: &mut Printer, written: &Variant, enclosing: Span) {
    printer.word(&written.name.text);
    match &written.payload {
        VariantPayload::None => {}
        VariantPayload::Tuple(types) => {
            printer.word("(");
            for (position, carried) in types.iter().enumerate() {
                if position > 0 {
                    printer.word(", ");
                }
                type_ref(printer, carried);
            }
            printer.word(")");
        }
        VariantPayload::Record(fields) => record_body(printer, fields, enclosing),
    }
}

/// `{`, one `name: Type` per line, then `}` — the one shape the grammar spreads over lines.
///
/// A record body serves both a type declaration and a variant's payload, and the two differ
/// only in what follows the `}`, so the `}` line is left open for the caller to end.
fn record_body(printer: &mut Printer, fields: &[RecordField], enclosing: Span) {
    open_block(printer);
    for field in fields {
        printer.comments_above(field.span);
        printer.open_line();
        printer.word(&field.name.text);
        printer.word(": ");
        type_ref(printer, &field.type_ref);
        printer.end_line();
    }
    close_block(printer, enclosing.end());
}

/// `trait Eq<T> { … }`, with one signature per line and no blank line between two of them.
pub(crate) fn trait_declaration(printer: &mut Printer, written: &TraitDeclaration) {
    printer.comments_above(written.span);
    printer.open_line();
    printer.word("trait ");
    printer.word(&written.name.text);
    written_over(printer, &written.parameter.text);
    open_block(printer);
    for declared in &written.methods {
        signature(printer, declared);
    }
    close_block(printer, written.span.end());
    printer.end_line();
}

fn signature(printer: &mut Printer, written: &Signature) {
    printer.comments_above(written.span);
    printer.open_line();
    printer.word("fn ");
    printer.word(&written.name.text);
    parameters(printer, &written.parameters);
    result_type(printer, written.result.as_ref());
    printer.end_line();
}

/// `instance Eq<Point> { … }`, with one blank line between two of the functions it writes.
fn instance(printer: &mut Printer, written: &InstanceDeclaration) {
    printer.comments_above(written.span);
    printer.open_line();
    printer.word("instance ");
    printer.word(&written.trait_name.text);
    written_over(printer, &written.for_type.text);
    open_block(printer);
    for (position, declared) in written.methods.iter().enumerate() {
        if position > 0 {
            printer.blank_line();
        }
        function(printer, declared);
    }
    close_block(printer, written.span.end());
    printer.end_line();
}

/// `derive Eq, Ord for User`: one line, whatever it names, because nothing in it can be broken.
fn derive(printer: &mut Printer, written: &DeriveDeclaration) {
    printer.comments_above(written.span);
    printer.open_line();
    printer.word("derive ");
    for (position, named) in written.traits.iter().enumerate() {
        if position > 0 {
            printer.word(", ");
        }
        printer.word(&named.text);
    }
    printer.word(" for ");
    printer.word(&written.for_type.text);
    printer.end_line();
}

/// `<T>`, which is what a trait is declared over and what an instance is written for.
fn written_over(printer: &mut Printer, name: &str) {
    printer.word("<");
    printer.word(name);
    printer.word(">");
}

fn function(printer: &mut Printer, written: &Function) {
    printer.comments_above(written.span);
    printer.open_line();
    printer.word("fn ");
    printer.word(&written.name.text);
    constrained_parameters(printer, &written.type_parameters);
    parameters(printer, &written.parameters);
    result_type(printer, written.result.as_ref());
    block(printer, &written.body, true);
    printer.end_line();
}

/// `(a: Int, b: Int)`, which a function and a trait's signature write the same way.
fn parameters(printer: &mut Printer, written: &[Parameter]) {
    printer.word("(");
    for (position, declared) in written.iter().enumerate() {
        if position > 0 {
            printer.word(", ");
        }
        parameter(printer, declared);
    }
    printer.word(")");
}

/// ` -> Bool`, which a declaration writes only where the author wrote one.
fn result_type(printer: &mut Printer, written: Option<&TypeRef>) {
    let Some(result) = written else {
        return;
    };
    printer.word(" -> ");
    type_ref(printer, result);
}

fn parameter(printer: &mut Printer, written: &Parameter) {
    printer.word(&written.name.text);
    let Some(declared) = &written.type_ref else {
        return;
    };
    printer.word(": ");
    type_ref(printer, declared);
}

/// `<T, E>`, which most declarations do not have.
fn type_parameters(printer: &mut Printer, parameters: &[Name]) {
    listed(printer, parameters.len(), |printer, position| {
        printer.word(&parameters[position].text);
    });
}

/// `<T>` or `<T: Eq<T>>`, which a function writes and a type declaration does not.
fn constrained_parameters(printer: &mut Printer, parameters: &[TypeParameter]) {
    listed(printer, parameters.len(), |printer, position| {
        let declared = &parameters[position];
        printer.word(&declared.name.text);
        let Some(constraint) = &declared.constraint else {
            return;
        };
        printer.word(": ");
        printer.word(&constraint.name.text);
        printer.word("<");
        type_ref(printer, &constraint.argument);
        printer.word(">");
    });
}

/// `<…>` around `count` type parameters, comma-separated, or nothing at all when there are none.
fn listed(printer: &mut Printer, count: usize, mut one: impl FnMut(&mut Printer, usize)) {
    if count == 0 {
        return;
    }
    printer.word("<");
    for position in 0..count {
        if position > 0 {
            printer.word(", ");
        }
        one(printer, position);
    }
    printer.word(">");
}

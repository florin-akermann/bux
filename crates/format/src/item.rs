//! The top-level declarations of a source file.

use lumen_ast::{Function, Import, Item, Parameter, Program, RecordField, TypeDeclaration};
use lumen_ast::{Name, Span, TypeDefinition, Variant, VariantPayload};

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
        Item::Function(declared) => function(printer, declared),
    }
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
    printer.word("type ");
    printer.word(&written.name.text);
    type_parameters(printer, &written.parameters);
    printer.word(" =");
    match &written.definition {
        TypeDefinition::Record(fields) => {
            record_body(printer, fields, written.span);
            printer.end_line();
        }
        TypeDefinition::Variants(variants) => variants_of(printer, variants, written.span),
    }
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

fn function(printer: &mut Printer, written: &Function) {
    printer.comments_above(written.span);
    printer.open_line();
    printer.word("fn ");
    printer.word(&written.name.text);
    type_parameters(printer, &written.type_parameters);
    printer.word("(");
    for (position, declared) in written.parameters.iter().enumerate() {
        if position > 0 {
            printer.word(", ");
        }
        parameter(printer, declared);
    }
    printer.word(")");
    if let Some(result) = &written.result {
        printer.word(" -> ");
        type_ref(printer, result);
    }
    block(printer, &written.body, true);
    printer.end_line();
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
    if parameters.is_empty() {
        return;
    }
    printer.word("<");
    for (position, parameter) in parameters.iter().enumerate() {
        if position > 0 {
            printer.word(", ");
        }
        printer.word(&parameter.text);
    }
    printer.word(">");
}

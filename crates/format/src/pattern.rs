//! Patterns as canonical form writes them.

use lumen_ast::{Name, Pattern, PatternKind};

use crate::literal::{integer, string};
use crate::printer::Printer;

/// One arm's pattern, always on one line.
pub(crate) fn pattern(printer: &mut Printer, written: &Pattern) {
    match &written.kind {
        PatternKind::Name(name) => printer.word(&name.text),
        PatternKind::Integer(value) => printer.word(&integer(*value)),
        PatternKind::String(value) => printer.word(&string(value)),
        PatternKind::Bool(value) => printer.word(if *value { "true" } else { "false" }),
        PatternKind::Tuple { name, elements } => carried(printer, &name.text, elements),
        PatternKind::Record { name, fields } => named_fields(printer, &name.text, fields),
    }
}

/// `Failed(reason)`, a variant matched on what it carries positionally.
fn carried(printer: &mut Printer, name: &str, elements: &[Pattern]) {
    printer.word(name);
    printer.word("(");
    for (position, element) in elements.iter().enumerate() {
        if position > 0 {
            printer.word(", ");
        }
        pattern(printer, element);
    }
    printer.word(")");
}

/// `Authorized { authorization_id }`, a variant matched on the fields it names.
fn named_fields(printer: &mut Printer, name: &str, fields: &[Name]) {
    printer.word(name);
    printer.word(" { ");
    for (position, field) in fields.iter().enumerate() {
        if position > 0 {
            printer.word(", ");
        }
        printer.word(&field.text);
    }
    printer.word(" }");
}

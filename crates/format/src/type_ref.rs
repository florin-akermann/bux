//! Types as canonical form writes them.

use lumen_ast::{TypeRef, TypeRefKind};

use crate::printer::Printer;

/// `Int`, `List<User>`, or `()`, always on one line.
pub(crate) fn type_ref(printer: &mut Printer, written: &TypeRef) {
    match &written.kind {
        TypeRefKind::Unit => printer.word("()"),
        TypeRefKind::Named { name, arguments } => {
            printer.word(&name.text);
            type_arguments(printer, arguments);
        }
    }
}

fn type_arguments(printer: &mut Printer, arguments: &[TypeRef]) {
    if arguments.is_empty() {
        return;
    }
    printer.word("<");
    for (position, argument) in arguments.iter().enumerate() {
        if position > 0 {
            printer.word(", ");
        }
        type_ref(printer, argument);
    }
    printer.word(">");
}

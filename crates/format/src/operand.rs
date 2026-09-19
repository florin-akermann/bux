//! Where parentheses go.
//!
//! The parser drops parentheses, keeping only the shape they produced, so the printer is what
//! puts back exactly the ones the shape needs. Every descent into a sub-expression goes through
//! here, which is what makes `parse(format(source))` equal `parse(source)`.

use lumen_ast::{Expr, ExprKind};

use crate::expr::{Records, binds, expression};
use crate::printer::Printer;

/// Writes `written` where an expression binding at least as tightly as `least` is expected.
///
/// Anything looser needs parentheses, and so does a record literal where the grammar reads a
/// `{` as the start of a block. Inside the parentheses both rules start over.
pub(crate) fn operand(printer: &mut Printer, written: &Expr, least: u8, records: Records) {
    if !needs_parentheses(written, least, records) {
        expression(printer, written, records);
        return;
    }
    printer.word("(");
    expression(printer, written, Records::Allowed);
    printer.word(")");
}

fn needs_parentheses(written: &Expr, least: u8, records: Records) -> bool {
    if records == Records::Forbidden && matches!(written.kind, ExprKind::Record { .. }) {
        return true;
    }
    binds(&written.kind) < least
}

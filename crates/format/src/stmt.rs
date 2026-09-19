//! Blocks, and the statements inside them.

use lumen_ast::{AssignOperator, Block, Expr, ForHeader, ForLoop, Mutability};
use lumen_ast::{Statement, StatementKind};

use crate::expr::Records;
use crate::operand::operand;
use crate::printer::Printer;

/// Writes `{`, the statements one per line, and the `}` that closes them.
///
/// The `}` line is left open, because an `if` continues it with an `else` and a statement ends
/// it with a newline. A `closed` of false leaves the `}` itself to the caller for the same
/// reason.
pub(crate) fn block(printer: &mut Printer, written: &Block, closed: bool) {
    open_block(printer);
    for statement in &written.statements {
        statement_line(printer, statement);
    }
    printer.comments_before(written.span.end());
    printer.dedent();
    printer.open_line();
    if closed {
        printer.word("}");
    }
}

/// Writes ` {` and opens the level the entries of a block are written at.
pub(crate) fn open_block(printer: &mut Printer) {
    printer.word(" {");
    printer.end_line();
    printer.indent();
}

/// Writes the `}` that closes a body, after any comment written before `end` and still pending.
pub(crate) fn close_block(printer: &mut Printer, end: usize) {
    printer.comments_before(end);
    printer.dedent();
    printer.open_line();
    printer.word("}");
}

fn statement_line(printer: &mut Printer, written: &Statement) {
    printer.comments_above(written.span);
    printer.open_line();
    statement(printer, written);
    printer.end_line();
}

fn statement(printer: &mut Printer, written: &Statement) {
    match &written.kind {
        StatementKind::Binding {
            mutability,
            name,
            value,
        } => {
            binding(printer, *mutability, &name.text, value);
        }
        StatementKind::Assign {
            target,
            operator,
            value,
        } => {
            operand(printer, target, 0, Records::Allowed);
            printer.word(match operator {
                AssignOperator::Set => " = ",
                AssignOperator::Add => " += ",
            });
            operand(printer, value, 0, Records::Allowed);
        }
        StatementKind::Return(None) => printer.word("return"),
        StatementKind::Return(Some(value)) => {
            printer.word("return ");
            operand(printer, value, 0, Records::Allowed);
        }
        StatementKind::Break => printer.word("break"),
        StatementKind::Continue => printer.word("continue"),
        StatementKind::For(loop_) => for_loop(printer, loop_),
        StatementKind::Expr(value) => operand(printer, value, 0, Records::Allowed),
    }
}

/// `total := 0` when the name is never assigned to again, `var total = 0` when it may be.
fn binding(printer: &mut Printer, mutability: Mutability, name: &str, value: &Expr) {
    let joiner = match mutability {
        Mutability::Immutable => " := ",
        Mutability::Mutable => {
            printer.word("var ");
            " = "
        }
    };
    printer.word(name);
    printer.word(joiner);
    operand(printer, value, 0, Records::Allowed);
}

fn for_loop(printer: &mut Printer, written: &ForLoop) {
    printer.word("for");
    match &written.header {
        ForHeader::Forever => {}
        ForHeader::While(condition) => {
            printer.word(" ");
            operand(printer, condition, 0, Records::Forbidden);
        }
        ForHeader::In { binding, iterable } => {
            printer.word(" ");
            printer.word(&binding.text);
            printer.word(" in ");
            operand(printer, iterable, 0, Records::Forbidden);
        }
    }
    block(printer, &written.body, true);
}

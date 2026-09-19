//! The two expressions that hold blocks: `if` and `match`.

use lumen_ast::{IfExpr, MatchExpr, Span};

use crate::expr::Records;
use crate::operand::operand;
use crate::pattern::pattern;
use crate::printer::Printer;
use crate::stmt::{block, close_block, open_block};

/// `if a {`, its block, and every `} else if b {` and `} else {` that follows on one line.
pub(crate) fn if_expr(printer: &mut Printer, chain: &IfExpr) {
    let mut branches = chain.branches.iter().enumerate().peekable();
    while let Some((position, branch)) = branches.next() {
        printer.word(if position > 0 { "} else if " } else { "if " });
        operand(printer, &branch.condition, 0, Records::Forbidden);
        let ends_the_chain = branches.peek().is_none() && chain.otherwise.is_none();
        block(printer, &branch.block, ends_the_chain);
    }
    let Some(otherwise) = &chain.otherwise else {
        return;
    };
    printer.word("} else");
    block(printer, otherwise, true);
}

/// `match scrutinee {`, then one `pattern => body` per line, then `}`.
pub(crate) fn match_expr(printer: &mut Printer, matched: &MatchExpr, span: Span) {
    printer.word("match ");
    operand(printer, &matched.scrutinee, 0, Records::Forbidden);
    open_block(printer);
    for arm in &matched.arms {
        printer.comments_above(arm.span);
        printer.open_line();
        pattern(printer, &arm.pattern);
        printer.word(" => ");
        operand(printer, &arm.body, 0, Records::Allowed);
        printer.end_line();
    }
    close_block(printer, span.end());
}

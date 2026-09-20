//! Expressions as canonical form writes them, always on one line.

use lumen_ast::{Arguments, BinaryOperator, Expr, ExprKind, FieldValue};
use lumen_ast::{NamedArgument, Path, UnaryOperator};

use crate::control::{if_expr, match_expr};
use crate::literal::{integer, string};
use crate::operand::operand;
use crate::printer::Printer;

/// Whether a `{` after a name would read as a record literal or as the start of a block.
///
/// The header of an `if`, a `for`, and a `match` forbids one, exactly as the parser does, so a
/// record literal written there is parenthesised.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Records {
    Allowed,
    Forbidden,
}

/// How tightly a postfix chain binds: tighter than any operator, looser than a bare value.
pub(crate) const POSTFIX: u8 = 6;

/// How tightly a value binds, which is tightly enough never to need parentheses.
const PRIMARY: u8 = 7;

/// A binding power nothing reaches, which is how an operand is made to keep its parentheses.
const PARENTHESISED: u8 = PRIMARY + 1;

/// How tightly a prefix operator binds.
const UNARY: u8 = 5;

/// The infix operators by precedence, loosest first, mirroring the parser's table.
const LEVELS: [&[(BinaryOperator, &str)]; 5] = [
    &[(BinaryOperator::Or, "||")],
    &[(BinaryOperator::And, "&&")],
    &[
        (BinaryOperator::Equal, "=="),
        (BinaryOperator::NotEqual, "!="),
        (BinaryOperator::Less, "<"),
        (BinaryOperator::LessOrEqual, "<="),
        (BinaryOperator::Greater, ">"),
        (BinaryOperator::GreaterOrEqual, ">="),
    ],
    &[(BinaryOperator::Add, "+"), (BinaryOperator::Subtract, "-")],
    &[
        (BinaryOperator::Multiply, "*"),
        (BinaryOperator::Divide, "/"),
        (BinaryOperator::Remainder, "%"),
    ],
];

/// The one level that does not chain, so that both its operands need parentheses.
const COMPARISON_LEVEL: u8 = 2;

pub(crate) fn expression(printer: &mut Printer, written: &Expr, records: Records) {
    match &written.kind {
        ExprKind::Name(name) => printer.word(&name.text),
        ExprKind::Integer(value) => printer.word(&integer(*value)),
        ExprKind::String(value) => printer.word(&string(value)),
        ExprKind::Bool(value) => printer.word(if *value { "true" } else { "false" }),
        ExprKind::Unit => printer.word("()"),
        ExprKind::Unary {
            operator,
            operand: inner,
        } => unary(printer, *operator, inner, records),
        ExprKind::Binary {
            operator,
            left,
            right,
        } => binary(
            printer,
            Infix {
                operator: *operator,
                left,
                right,
            },
            records,
        ),
        ExprKind::Call { callee, arguments } => call(printer, callee, arguments, records),
        ExprKind::Field { receiver, name } => {
            operand(printer, receiver, POSTFIX, records);
            printer.word(".");
            printer.word(&name.text);
        }
        ExprKind::Try(inner) => {
            operand(printer, inner, POSTFIX, records);
            printer.word("?");
        }
        ExprKind::List(elements) => written_list(printer, elements),
        ExprKind::Record { base, fields } => record(printer, base, fields),
        ExprKind::If(chain) => if_expr(printer, chain),
        ExprKind::Match(matched) => match_expr(printer, matched, written.span),
    }
}

/// How tightly an expression binds, which is what decides whether it needs parentheses.
pub(crate) fn binds(kind: &ExprKind) -> u8 {
    match kind {
        ExprKind::Binary { operator, .. } => level_of(*operator),
        ExprKind::Unary { .. } => UNARY,
        ExprKind::Call { .. } | ExprKind::Field { .. } | ExprKind::Try(_) => POSTFIX,
        _ => PRIMARY,
    }
}

fn unary(printer: &mut Printer, operator: UnaryOperator, inner: &Expr, records: Records) {
    printer.word(match operator {
        UnaryOperator::Not => "!",
        UnaryOperator::Negate => "-",
    });
    operand(printer, inner, least_after(operator, inner), records);
}

/// How tightly the operand of a prefix operator must bind to be written without parentheses.
///
/// `docs/specs/grammar.md` reads a `-` before a number as part of that number, so `-(7.abs())`
/// written as `-7.abs()` would come back as `(-7).abs()`, which is a different program.
/// An operand whose first character is a digit therefore keeps its parentheses after a `-`.
fn least_after(operator: UnaryOperator, inner: &Expr) -> u8 {
    if operator == UnaryOperator::Negate && opens_with_a_number(inner) {
        return PARENTHESISED;
    }
    UNARY
}

/// Whether the first thing written for this expression is the digits of a number.
///
/// A negative number is written `-7` and so opens with a `-`, which the grammar reads as the
/// second of two minus signs rather than as part of a number.
fn opens_with_a_number(written: &Expr) -> bool {
    match &written.kind {
        ExprKind::Integer(value) => !value.is_negative(),
        ExprKind::Call { callee, .. } => opens_with_a_number(callee),
        ExprKind::Field { receiver, .. } => opens_with_a_number(receiver),
        ExprKind::Try(inner) => opens_with_a_number(inner),
        _ => false,
    }
}

/// The three parts of an infix expression, which is one thing and not three arguments.
#[derive(Clone, Copy)]
struct Infix<'a> {
    operator: BinaryOperator,
    left: &'a Expr,
    right: &'a Expr,
}

/// `left op right`, with one space around the operator and parentheses only where needed.
fn binary(printer: &mut Printer, infix: Infix, records: Records) {
    let level = level_of(infix.operator);
    let leftmost = if level == COMPARISON_LEVEL {
        level + 1
    } else {
        level
    };
    operand(printer, infix.left, leftmost, records);
    printer.word(" ");
    printer.word(text_of(infix.operator));
    printer.word(" ");
    operand(printer, infix.right, level + 1, records);
}

fn call(printer: &mut Printer, callee: &Expr, arguments: &Arguments, records: Records) {
    operand(printer, callee, POSTFIX, records);
    printer.word("(");
    match arguments {
        Arguments::Positional(passed) => values(printer, passed),
        Arguments::Named(written) => named(printer, written),
    }
    printer.word(")");
}

/// `[first, second]`, and `[]`, which are spaced inside their brackets by nothing.
fn written_list(printer: &mut Printer, elements: &[Expr]) {
    printer.word("[");
    values(printer, elements);
    printer.word("]");
}

/// `old, new`: values in order, each written after the one before it with `, ` between.
fn values(printer: &mut Printer, written: &[Expr]) {
    for (position, value) in written.iter().enumerate() {
        if position > 0 {
            printer.word(", ");
        }
        operand(printer, value, 0, Records::Allowed);
    }
}

/// `from: old, to: new`: each value with the parameter it is passed for, spaced as a field is.
fn named(printer: &mut Printer, written: &[NamedArgument]) {
    for (position, argument) in written.iter().enumerate() {
        if position > 0 {
            printer.word(", ");
        }
        printer.word(&argument.name.text);
        printer.word(": ");
        operand(printer, &argument.value, 0, Records::Allowed);
    }
}

/// `User { id: id }`, or `User {}` when it names no field.
fn record(printer: &mut Printer, base: &Path, fields: &[FieldValue]) {
    printer.path(base);
    if fields.is_empty() {
        printer.word(" {}");
        return;
    }
    printer.word(" { ");
    for (position, field) in fields.iter().enumerate() {
        if position > 0 {
            printer.word(", ");
        }
        printer.word(&field.name.text);
        printer.word(": ");
        operand(printer, &field.value, 0, Records::Allowed);
    }
    printer.word(" }");
}

fn level_of(operator: BinaryOperator) -> u8 {
    let found = LEVELS
        .iter()
        .position(|level| level.iter().any(|(candidate, _)| *candidate == operator));
    u8::try_from(found.expect("every operator sits on a level")).expect("five levels fit in a byte")
}

fn text_of(operator: BinaryOperator) -> &'static str {
    LEVELS
        .iter()
        .flat_map(|level| level.iter())
        .find(|(candidate, _)| *candidate == operator)
        .expect("every operator spells something")
        .1
}

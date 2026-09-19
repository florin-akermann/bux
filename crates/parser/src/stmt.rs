//! Blocks, and the statements inside them.

use lumen_ast::{
    AssignOperator, Block, Expr, ExprKind, ForHeader, ForLoop, Mutability, Name, Statement,
    StatementKind,
};
use lumen_lexer::{Keyword, Punct, TokenKind};

use crate::cursor::Cursor;
use crate::error::{Expected, ParseError, ParseErrorKind};
use crate::expr::{RecordLiterals, expression};
use crate::list::newline_separated;

fn statement(cursor: &mut Cursor) -> Result<Statement, ParseError> {
    let start = cursor.offset();
    let kind = statement_kind(cursor)?;
    Ok(Statement {
        kind,
        span: cursor.span_since(start),
    })
}

fn statement_kind(cursor: &mut Cursor) -> Result<StatementKind, ParseError> {
    match cursor.peek_kind(0) {
        Some(TokenKind::Keyword(Keyword::Var)) => binding(cursor),
        Some(TokenKind::Keyword(Keyword::Return)) => return_statement(cursor),
        Some(TokenKind::Keyword(Keyword::Break)) => Ok(single(cursor, StatementKind::Break)),
        Some(TokenKind::Keyword(Keyword::Continue)) => Ok(single(cursor, StatementKind::Continue)),
        Some(TokenKind::Keyword(Keyword::For)) => for_statement(cursor),
        Some(TokenKind::Identifier) if cursor.peek_kind(1) == Some(WALRUS) => binding(cursor),
        Some(UNDERSCORE) => discard(cursor),
        _ => expression_statement(cursor),
    }
}

const WALRUS: TokenKind = TokenKind::Punct(Punct::Walrus);

const UNDERSCORE: TokenKind = TokenKind::Keyword(Keyword::Underscore);

/// `_ = save(user)`, which works the value out and throws it away on purpose.
///
/// `_` is no name, so `=` is all that may follow it: `_ += 1` and a bare `_` are refused here
/// rather than carried on as something the later phases would have to refuse again.
fn discard(cursor: &mut Cursor) -> Result<StatementKind, ParseError> {
    cursor.expect_keyword(Keyword::Underscore)?;
    cursor.expect_punct(Punct::Eq)?;
    Ok(StatementKind::Discard(expression(
        cursor,
        RecordLiterals::Allowed,
    )?))
}

/// A statement that is one keyword and nothing else.
fn single(cursor: &mut Cursor, kind: StatementKind) -> StatementKind {
    cursor.advance();
    kind
}

/// `total := 0` binds a name never assigned to again; `var total = 0` binds one that may be.
///
/// `var` introduces the binding and `:=` joins it, so which keyword opened it decides which
/// token joins it: `var total := 0` is as much an error as `total = 0` in this position.
fn binding(cursor: &mut Cursor) -> Result<StatementKind, ParseError> {
    let (mutability, joiner) = if cursor.eat_keyword(Keyword::Var).is_some() {
        (Mutability::Mutable, Punct::Eq)
    } else {
        (Mutability::Immutable, Punct::Walrus)
    };
    let name = cursor.expect_name(Expected::Name)?;
    cursor.expect_punct(joiner)?;
    Ok(StatementKind::Binding {
        mutability,
        name,
        value: expression(cursor, RecordLiterals::Allowed)?,
    })
}

fn return_statement(cursor: &mut Cursor) -> Result<StatementKind, ParseError> {
    cursor.expect_keyword(Keyword::Return)?;
    if cursor.at_statement_end() {
        return Ok(StatementKind::Return(None));
    }
    let value = expression(cursor, RecordLiterals::Allowed)?;
    Ok(StatementKind::Return(Some(value)))
}

fn for_statement(cursor: &mut Cursor) -> Result<StatementKind, ParseError> {
    cursor.expect_keyword(Keyword::For)?;
    let header = for_header(cursor)?;
    let body = block(cursor)?;
    Ok(StatementKind::For(Box::new(ForLoop { header, body })))
}

/// What a `for` loops over: a collection, a condition, or nothing at all.
fn for_header(cursor: &mut Cursor) -> Result<ForHeader, ParseError> {
    if cursor.at_punct(Punct::LBrace) {
        return Ok(ForHeader::Forever);
    }
    if cursor.peek_kind(0) == Some(TokenKind::Identifier) && cursor.peek_kind(1) == Some(IN) {
        let binding = cursor.expect_name(Expected::Name)?;
        cursor.expect_keyword(Keyword::In)?;
        let iterable = expression(cursor, RecordLiterals::Forbidden)?;
        return Ok(ForHeader::In { binding, iterable });
    }
    Ok(ForHeader::While(expression(
        cursor,
        RecordLiterals::Forbidden,
    )?))
}

const IN: TokenKind = TokenKind::Keyword(Keyword::In);

pub(crate) fn block(cursor: &mut Cursor) -> Result<Block, ParseError> {
    let start = cursor.offset();
    cursor.expect_punct(Punct::LBrace)?;
    let statements = cursor.nested(|cursor| newline_separated(cursor, Punct::RBrace, statement))?;
    Ok(Block {
        statements,
        span: cursor.span_since(start),
    })
}

/// An expression, which becomes an assignment when `=` or `+=` follows it.
fn expression_statement(cursor: &mut Cursor) -> Result<StatementKind, ParseError> {
    let written = expression(cursor, RecordLiterals::Allowed)?;
    let Some(operator) = eat_assign_operator(cursor) else {
        return Ok(StatementKind::Expr(written));
    };
    Ok(StatementKind::Assign {
        target: assigned_to(written)?,
        operator,
        value: expression(cursor, RecordLiterals::Allowed)?,
    })
}

/// The name an assignment changes, which is all the left of `=` or `+=` may be.
///
/// `user.name = "Bob"` reaches inside a value, and a value is changed by building the one it
/// becomes, so anything but a bare name is refused here rather than carried further.
fn assigned_to(written: Expr) -> Result<Name, ParseError> {
    match written.kind {
        ExprKind::Name(name) => Ok(name),
        _ => Err(ParseError::new(
            ParseErrorKind::AssignedToValue,
            written.span,
        )),
    }
}

fn eat_assign_operator(cursor: &mut Cursor) -> Option<AssignOperator> {
    if cursor.eat_punct(Punct::Eq).is_some() {
        return Some(AssignOperator::Set);
    }
    cursor.eat_punct(Punct::PlusEq).map(|_| AssignOperator::Add)
}

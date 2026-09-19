//! The two expressions that hold blocks: `if` and `match`.

use lumen_ast::{Branch, IfExpr, MatchArm, MatchExpr};
use lumen_lexer::{Keyword, Punct, TokenKind};

use crate::cursor::Cursor;
use crate::error::ParseError;
use crate::expr::{RecordLiterals, expression};
use crate::list::newline_separated;
use crate::pattern::pattern;
use crate::stmt::block;

pub(crate) fn if_expression(cursor: &mut Cursor) -> Result<IfExpr, ParseError> {
    let mut branches = vec![branch(cursor)?];
    let mut otherwise = None;
    while cursor.eat_keyword(Keyword::Else).is_some() {
        if cursor.at(TokenKind::Keyword(Keyword::If)) {
            branches.push(branch(cursor)?);
            continue;
        }
        otherwise = Some(block(cursor)?);
        break;
    }
    Ok(IfExpr {
        branches,
        otherwise,
    })
}

/// One `if condition { … }`, whether it opens the chain or follows an `else`.
fn branch(cursor: &mut Cursor) -> Result<Branch, ParseError> {
    let start = cursor.offset();
    cursor.expect_keyword(Keyword::If)?;
    let condition = expression(cursor, RecordLiterals::Forbidden)?;
    let block = block(cursor)?;
    Ok(Branch {
        condition,
        block,
        span: cursor.span_since(start),
    })
}

pub(crate) fn match_expression(cursor: &mut Cursor) -> Result<MatchExpr, ParseError> {
    cursor.expect_keyword(Keyword::Match)?;
    let scrutinee = expression(cursor, RecordLiterals::Forbidden)?;
    cursor.expect_punct(Punct::LBrace)?;
    let arms = newline_separated(cursor, Punct::RBrace, match_arm)?;
    Ok(MatchExpr { scrutinee, arms })
}

fn match_arm(cursor: &mut Cursor) -> Result<MatchArm, ParseError> {
    let start = cursor.offset();
    let pattern = pattern(cursor)?;
    cursor.expect_punct(Punct::FatArrow)?;
    let body = expression(cursor, RecordLiterals::Allowed)?;
    Ok(MatchArm {
        pattern,
        body,
        span: cursor.span_since(start),
    })
}

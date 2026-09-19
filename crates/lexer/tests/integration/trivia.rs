//! Newlines and comments are tokens; blanks separate tokens and are not.

use lumen_lexer::{Span, TokenKind, lex};

use crate::common::{kinds, texts};

#[test]
fn a_newline_is_a_token() {
    assert_eq!(
        kinds("a\nb"),
        [
            TokenKind::Identifier,
            TokenKind::Newline,
            TokenKind::Identifier
        ]
    );
}

#[test]
fn consecutive_newlines_are_one_token_each() {
    assert_eq!(kinds("\n\n"), [TokenKind::Newline, TokenKind::Newline]);
}

#[test]
fn blanks_separate_tokens_and_are_not_tokens() {
    let tokens = lex("a \t\r b");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].span, Span::new(0, 1));
    assert_eq!(tokens[1].span, Span::new(5, 1));
}

#[test]
fn a_carriage_return_before_a_newline_is_a_blank() {
    assert_eq!(kinds("a\r\nb").len(), 3);
    assert_eq!(kinds("a\r\nb")[1], TokenKind::Newline);
}

#[test]
fn a_line_comment_runs_to_the_end_of_the_line_excluding_the_newline() {
    let source = "x // note\ny";
    assert_eq!(
        kinds(source),
        [
            TokenKind::Identifier,
            TokenKind::LineComment,
            TokenKind::Newline,
            TokenKind::Identifier
        ]
    );
    assert_eq!(texts(source)[1], "// note");
}

#[test]
fn a_comment_on_a_crlf_line_excludes_the_carriage_return() {
    assert_eq!(texts("// note\r\nx")[0], "// note");
    assert_eq!(kinds("// note\r\nx").len(), 3);
}

#[test]
fn a_comment_at_the_end_of_input_needs_no_newline() {
    assert_eq!(kinds("// note"), [TokenKind::LineComment]);
    assert_eq!(texts("// note"), ["// note"]);
}

#[test]
fn a_lone_slash_is_division_and_not_a_comment() {
    assert_eq!(
        kinds("a / b"),
        [
            TokenKind::Identifier,
            TokenKind::Punct(lumen_lexer::Punct::Slash),
            TokenKind::Identifier
        ]
    );
}

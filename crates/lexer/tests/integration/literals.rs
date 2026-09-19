//! Integer and string literals, including the unterminated string that keeps lexing total.

use lumen_lexer::{Punct, Span, TokenKind, lex};

use crate::common::{kinds, texts};

#[test]
fn a_run_of_digits_is_an_integer() {
    assert_eq!(kinds("42"), [TokenKind::Integer]);
    assert_eq!(texts("42 007"), ["42", "007"]);
}

#[test]
fn a_run_of_digits_glued_to_letters_is_an_integer_then_an_identifier() {
    assert_eq!(kinds("123abc"), [TokenKind::Integer, TokenKind::Identifier]);
    assert_eq!(texts("123abc"), ["123", "abc"]);
}

#[test]
fn a_minus_is_punctuation_and_not_part_of_the_integer() {
    assert_eq!(
        kinds("-1"),
        [TokenKind::Punct(Punct::Minus), TokenKind::Integer]
    );
}

#[test]
fn a_quoted_run_is_a_string() {
    assert_eq!(kinds(r#""hello""#), [TokenKind::String]);
    assert_eq!(lex(r#""hello""#)[0].span, Span::new(0, 7));
}

#[test]
fn an_escaped_quote_does_not_close_a_string() {
    assert_eq!(kinds(r#""a\"b""#), [TokenKind::String]);
    assert_eq!(texts(r#""a\"b""#), [r#""a\"b""#]);
}

#[test]
fn any_character_may_be_escaped_and_the_lexer_does_not_judge_it() {
    assert_eq!(kinds(r#""\q\\""#), [TokenKind::String]);
}

#[test]
fn a_string_ends_at_the_first_unescaped_quote() {
    assert_eq!(texts(r#""a" "b""#), [r#""a""#, r#""b""#]);
}

#[test]
fn a_string_may_be_empty() {
    assert_eq!(kinds(r#""""#), [TokenKind::String]);
}

#[test]
fn a_string_without_a_closing_quote_is_unterminated_up_to_the_newline() {
    let source = "\"abc\nx";
    assert_eq!(
        kinds(source),
        [
            TokenKind::UnterminatedString,
            TokenKind::Newline,
            TokenKind::Identifier
        ]
    );
    assert_eq!(lex(source)[0].span, Span::new(0, 4));
}

#[test]
fn a_string_without_a_closing_quote_at_the_end_of_input_is_unterminated() {
    assert_eq!(kinds("\"abc"), [TokenKind::UnterminatedString]);
    assert_eq!(lex("\"abc")[0].span, Span::new(0, 4));
}

#[test]
fn an_unterminated_string_on_a_crlf_line_stops_before_the_carriage_return() {
    assert_eq!(texts("\"abc\r\nx")[0], "\"abc");
    assert_eq!(kinds("\"abc\r\nx")[1], TokenKind::Newline);
}

#[test]
fn a_backslash_before_a_newline_does_not_escape_the_newline() {
    assert_eq!(
        kinds("\"a\\\nb"),
        [
            TokenKind::UnterminatedString,
            TokenKind::Newline,
            TokenKind::Identifier
        ]
    );
}

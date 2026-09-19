//! Helpers shared by the lexer's behaviour tests.

use lumen_lexer::{Token, TokenKind, lex};

/// The kinds of the tokens `source` lexes to, in order.
pub fn kinds(source: &str) -> Vec<TokenKind> {
    lex(source).into_iter().map(|token| token.kind).collect()
}

/// The source text of each token `source` lexes to, in order.
pub fn texts(source: &str) -> Vec<&str> {
    lex(source)
        .into_iter()
        .map(|token| token.span.text(source))
        .collect()
}

/// One `<kind> <start>..<end> <text>` line per token, the format of a `.tokens` example file.
pub fn render(source: &str) -> String {
    lex(source)
        .into_iter()
        .map(|token| render_token(source, token))
        .collect()
}

fn render_token(source: &str, token: Token) -> String {
    format!(
        "{:?} {}..{} {:?}\n",
        token.kind,
        token.span.start(),
        token.span.end(),
        token.span.text(source)
    )
}

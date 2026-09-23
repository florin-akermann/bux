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
///
/// A keyword and a punctuation are each named by the text that spells them, as the Bux lexer
/// names them, so the runner under `tests/` reads the same file.
pub fn render(source: &str) -> String {
    lex(source)
        .into_iter()
        .map(|token| render_token(source, token))
        .collect()
}

fn render_token(source: &str, token: Token) -> String {
    let text = token.span.text(source);
    let kind = match token.kind {
        TokenKind::Keyword(_) => format!("Keyword({text})"),
        TokenKind::Punct(_) => format!("Punct({text})"),
        other => format!("{other:?}"),
    };
    format!(
        "{kind} {}..{} \"{}\"\n",
        token.span.start(),
        token.span.end(),
        escaped(text)
    )
}

/// `text` with a quote, a backslash, a line break, a tab, and a carriage return escaped, and
/// every other character as it is, which is how the runner quotes the text of a token.
fn escaped(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '"' => "\\\"".to_owned(),
            '\\' => "\\\\".to_owned(),
            '\n' => "\\n".to_owned(),
            '\t' => "\\t".to_owned(),
            '\r' => "\\r".to_owned(),
            other => other.to_string(),
        })
        .collect()
}

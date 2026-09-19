//! Literals, written back into the text that spells them.
//!
//! The parser decodes a literal, so the printer is what spells it again. Re-lexing what this
//! writes gives back the same value, which is what makes the round trip hold.

/// A whole number, which carries its own sign when it is negative.
pub(crate) fn integer(value: i64) -> String {
    value.to_string()
}

/// A string, with the five escapes `docs/specs/grammar.md` knows written back in.
pub(crate) fn string(value: &str) -> String {
    let mut spelled = String::with_capacity(value.len() + 2);
    spelled.push('"');
    for character in value.chars() {
        match character {
            '"' => spelled.push_str("\\\""),
            '\\' => spelled.push_str("\\\\"),
            '\n' => spelled.push_str("\\n"),
            '\t' => spelled.push_str("\\t"),
            '\r' => spelled.push_str("\\r"),
            plain => spelled.push(plain),
        }
    }
    spelled.push('"');
    spelled
}

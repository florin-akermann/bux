//! The data form of a diagnostic, field for field, as `docs/specs/diagnostics.md` states it.

use lumen_diagnostics::{Code, Diagnostic, Fix, json};
use lumen_lexer::Span;

use crate::common::{FILE, chained};

#[test]
fn a_diagnostic_is_one_object_of_the_file_the_code_the_message_and_the_span() {
    assert_eq!(
        json(&chained(Span::new(13, 9), None), FILE),
        concat!(
            r#"{"file":"demo.lm","code":"L0105","#,
            r#""message":"comparisons do not chain","#,
            r#""span":{"start":13,"len":9}}"#,
            "\n"
        )
    );
}

#[test]
fn advice_is_a_field_of_its_own_when_there_is_any() {
    let written = json(&chained(Span::new(13, 9), Some("compare twice")), FILE);

    assert!(written.contains(r#","help":"compare twice"}"#), "{written}");
}

#[test]
fn an_edit_is_where_it_goes_and_what_goes_there() {
    let fixed =
        chained(Span::new(13, 9), None).fixed_by(Fix::new(Span::new(0, 4), "ab".to_owned()));

    let written = json(&fixed, FILE);

    assert!(
        written.contains(r#","fix":{"start":0,"len":4,"text":"ab"}}"#),
        "{written}"
    );
}

#[test]
fn a_quote_a_backslash_and_a_newline_are_each_written_the_way_json_writes_them() {
    let awkward = Diagnostic::new(
        Code::ChainedComparison,
        "a \" a \\ a \n a \t end".to_owned(),
        Span::new(0, 1),
        None,
    );

    let written = json(&awkward, FILE);

    assert!(
        written.contains(r#""message":"a \" a \\ a \n a \t end""#),
        "{written}"
    );
}

#[test]
fn a_control_character_is_written_as_the_four_digits_json_asks_for() {
    let awkward = Diagnostic::new(
        Code::ChainedComparison,
        "a \u{1} end".to_owned(),
        Span::new(0, 1),
        None,
    );

    assert!(
        json(&awkward, FILE).contains(r#""message":"a \u0001 end""#),
        "{}",
        json(&awkward, FILE)
    );
}

#[test]
fn a_data_form_is_one_line_even_when_the_text_it_holds_is_not() {
    let over_two_lines = Diagnostic::new(
        Code::ChainedComparison,
        "over\ntwo lines".to_owned(),
        Span::new(0, 1),
        Some("advice\nover two lines".to_owned()),
    );

    let written = json(&over_two_lines, FILE);

    assert_eq!(written.matches('\n').count(), 1, "{written:?}");
    assert!(written.ends_with("}\n"), "{written:?}");
}

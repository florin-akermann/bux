//! The catalogue: one home per code, and no code written twice.

use std::collections::HashSet;

use lumen_diagnostics::{CODES, Code};

#[test]
fn every_code_is_written_as_an_l_and_four_digits() {
    for &code in CODES {
        let number = code.number();
        assert_eq!(number.len(), 5, "{code:?}");
        assert!(number.starts_with('L'), "{code:?}");
        assert!(
            number[1..].chars().all(|digit| digit.is_ascii_digit()),
            "{code:?}"
        );
    }
}

#[test]
fn no_two_codes_are_written_the_same_way() {
    let numbers: HashSet<&str> = CODES.iter().map(|code| code.number()).collect();
    assert_eq!(numbers.len(), CODES.len());
}

#[test]
fn a_code_is_found_by_the_way_it_is_written() {
    for &code in CODES {
        assert_eq!(Code::written_as(code.number()), Some(code), "{code:?}");
    }
    assert_eq!(Code::written_as("L9999"), None);
    assert_eq!(Code::written_as("l0100"), None);
    assert_eq!(Code::written_as(""), None);
}

#[test]
fn each_phase_of_the_compiler_has_its_own_range_of_codes() {
    assert_eq!(Code::ChainedComparison.number(), "L0105");
    assert_eq!(Code::NotCanonical.number(), "L0200");
    assert_eq!(Code::UnresolvedName.number(), "L0300");
}

#[test]
fn every_explanation_names_its_own_code_in_its_heading() {
    for &code in CODES {
        let heading = code
            .explanation()
            .lines()
            .next()
            .expect("an explanation has a heading");
        assert!(
            heading.starts_with(&format!("# {}", code.number())),
            "{heading}"
        );
    }
}

#[test]
fn every_explanation_says_more_than_a_help_line_has_room_for() {
    for &code in CODES {
        assert!(code.explanation().len() > 200, "{code:?}");
    }
}

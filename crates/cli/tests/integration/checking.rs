//! `lumen check`: the refusal as a page to read, and the same refusal as data.

use crate::common::{Example, Run, lumen};

/// A module whose second line is not what canonical form writes there.
const NOT_CANONICAL: &str = "fn answer() -> Int {\n     7\n}\n";

/// The same module written the way canonical form writes it.
const CANONICAL: &str = "fn answer() -> Int {\n    7\n}\n";

#[test]
fn check_says_nothing_about_a_file_the_compiler_accepts() {
    let example = Example::new(CANONICAL);

    let run = as_a_page(&example);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "");
    assert_eq!(run.stderr, "");
}

#[test]
fn a_file_the_compiler_accepts_is_nothing_at_all_as_data_either() {
    let example = Example::new(CANONICAL);

    let run = as_data(&example);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "");
}

#[test]
fn the_data_form_goes_to_standard_output_and_leaves_standard_error_empty() {
    let example = Example::new(NOT_CANONICAL);

    let run = as_data(&example);

    assert_eq!(run.code, 1);
    assert_eq!(run.stderr, "");
    assert!(run.stdout.starts_with('{'), "{}", run.stdout);
    assert!(run.stdout.ends_with("}\n"), "{}", run.stdout);
}

#[test]
fn the_data_form_names_the_file_the_code_the_message_and_the_span() {
    let example = Example::new(NOT_CANONICAL);
    let path = example.path.to_str().expect("a UTF-8 path").to_owned();

    let run = as_data(&example);

    for named in [
        format!(r#""file":"{path}""#),
        r#""code":"L0200""#.to_owned(),
        r#""message":"this line is not in canonical form""#.to_owned(),
        r#""span":{"start":21,"len":6}"#.to_owned(),
    ] {
        assert!(run.stdout.contains(&named), "{named} is in {}", run.stdout);
    }
}

#[test]
fn a_file_that_departs_from_canonical_form_carries_the_canonical_text_as_its_edit() {
    let example = Example::new(NOT_CANONICAL);

    let run = as_data(&example);

    assert!(
        run.stdout
            .contains(r#""fix":{"start":0,"len":30,"text":"fn answer() -> Int {\n    7\n}\n"}"#),
        "{}",
        run.stdout
    );
}

#[test]
fn a_refusal_the_compiler_has_no_edit_for_carries_none() {
    let example = Example::new("fn answer() -> Int {\n    missing()\n}\n");

    let run = as_data(&example);

    assert_eq!(run.code, 1);
    assert!(run.stdout.contains(r#""code":"L0300""#), "{}", run.stdout);
    assert!(!run.stdout.contains(r#""fix""#), "{}", run.stdout);
}

#[test]
fn without_the_flag_the_refusal_is_the_page_a_reader_sees_on_standard_error() {
    let example = Example::new(NOT_CANONICAL);

    let run = as_a_page(&example);

    assert_eq!(run.code, 1);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.starts_with("error[L0200]: "), "{}", run.stderr);
}

#[test]
fn the_help_of_check_says_what_the_json_flag_writes() {
    let run = lumen(&["help", "check"]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.contains("--json"), "{}", run.stdout);
}

/// Checking `example` and asking for the refusal as data.
fn as_data(example: &Example) -> Run {
    lumen(&["check", "--json", path_of(example)])
}

/// Checking `example` and taking the refusal as the page a reader sees.
fn as_a_page(example: &Example) -> Run {
    lumen(&["check", path_of(example)])
}

fn path_of(example: &Example) -> &str {
    example.path.to_str().expect("a UTF-8 path")
}

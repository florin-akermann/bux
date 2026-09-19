//! The layout of a rendered diagnostic, line for line.

use crate::common::rendered;

#[test]
fn a_diagnostic_is_the_message_the_place_the_source_and_the_help() {
    assert_eq!(
        rendered(
            "fn f() {\n    a < b < c\n}\n",
            "a < b < c",
            Some("compare twice and join the two with `&&`")
        ),
        concat!(
            "error[L0105]: comparisons do not chain\n",
            "  --> demo.lm:2:5\n",
            "\n",
            "  2 |     a < b < c\n",
            "    |     ^^^^^^^^^\n",
            "\n",
            "help: compare twice and join the two with `&&`\n",
        )
    );
}

#[test]
fn a_diagnostic_with_nothing_to_advise_ends_at_the_carets() {
    assert_eq!(
        rendered("fn f() {\n    a < b < c\n}\n", "a < b < c", None),
        concat!(
            "error[L0105]: comparisons do not chain\n",
            "  --> demo.lm:2:5\n",
            "\n",
            "  2 |     a < b < c\n",
            "    |     ^^^^^^^^^\n",
        )
    );
}

#[test]
fn a_span_that_ends_on_a_later_line_is_shown_on_the_line_it_starts_on() {
    assert_eq!(
        rendered(
            "fn f() {\n    a <\n        b < c\n}\n",
            "a <\n        b < c",
            None
        ),
        concat!(
            "error[L0105]: comparisons do not chain\n",
            "  --> demo.lm:2:5\n",
            "\n",
            "  2 |     a <\n",
            "    |     ^^^ this runs on to line 3\n",
        )
    );
}

#[test]
fn the_gutter_grows_with_the_line_number() {
    let source = "// 1\n".repeat(9) + "fn f() {\n    a < b\n}\n";
    assert!(
        rendered(&source, "a < b", None).contains("  11 |     a < b\n     |     ^^^^^\n"),
        "{}",
        rendered(&source, "a < b", None)
    );
}

#[test]
fn a_column_counts_characters_rather_than_bytes() {
    assert_eq!(
        rendered("// é é\nfn f() {\n}\n", "é\n", None)
            .lines()
            .nth(1),
        Some("  --> demo.lm:1:6")
    );
}

#[test]
fn the_first_line_of_a_file_is_line_one() {
    assert_eq!(
        rendered("import io\n", "import", None).lines().nth(1),
        Some("  --> demo.lm:1:1")
    );
}

#[test]
fn a_tab_in_the_line_is_a_tab_in_the_caret_row_so_the_two_line_up() {
    let rendered = rendered("fn f() {\n\ta < b < c\n}\n", "b < c", None);
    let rows: Vec<&str> = rendered.lines().skip(3).collect();

    assert_eq!(rows, ["  2 | \ta < b < c", "    | \t    ^^^^^"]);
}

#[test]
fn a_span_past_the_end_of_the_file_points_at_its_last_line() {
    let source = "import io\n";
    assert_eq!(
        rendered(source, "io\n", None).lines().nth(1),
        Some("  --> demo.lm:1:8")
    );
}

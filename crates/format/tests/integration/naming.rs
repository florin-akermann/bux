//! `docs/specs/naming.md`: how canonical form spells a declared name.

use lumen_diagnostics::render;
use lumen_format::check;

/// The diagnostic `check` refuses `source` with, rendered as the author sees it.
fn refusal(source: &str) -> String {
    match check(source) {
        Ok(()) => panic!("{source:?} is canonical"),
        Err(error) => render(&error.diagnostic(), source, "demo.lm"),
    }
}

/// The message line alone of the diagnostic `source` is refused with.
fn message(source: &str) -> String {
    refusal(source)
        .lines()
        .next()
        .expect("a rendered diagnostic opens with its message")
        .to_owned()
}

/// The help line alone of the diagnostic `source` is refused with.
fn help(source: &str) -> String {
    refusal(source)
        .lines()
        .find_map(|line| line.strip_prefix("help: "))
        .expect("a naming refusal has something to advise")
        .to_owned()
}

#[test]
fn a_function_written_in_anything_but_snake_case_is_refused_with_the_spelling_it_wants() {
    assert_eq!(
        help("fn countActive(users: Int) -> Int {\n    users\n}\n"),
        "canonical form spells this name `count_active`"
    );
}

#[test]
fn a_type_written_in_anything_but_pascal_case_is_refused_with_the_spelling_it_wants() {
    assert_eq!(
        help("type user_id = user_id(Int)\n"),
        "canonical form spells this name `UserId`"
    );
}

#[test]
fn an_acronym_is_a_word_so_a_type_spelled_in_capitals_does_not_compile() {
    assert_eq!(
        help("type UserID = UserID(Int)\n"),
        "canonical form spells this name `UserId`"
    );
}

#[test]
fn an_acronym_that_runs_into_the_word_after_it_is_split_where_that_word_begins() {
    assert_eq!(
        help("type HTTPServer = HTTPServer(Int)\n"),
        "canonical form spells this name `HttpServer`"
    );
}

#[test]
fn a_digit_continues_the_word_it_is_written_in() {
    assert!(check("type Int32 = Int32(Int)\n").is_ok());
}

#[test]
fn a_parameter_is_snake_case_because_a_call_writes_its_name() {
    assert_eq!(
        help("fn shout(theWord: String) -> String {\n    theWord\n}\n"),
        "canonical form spells this name `the_word`"
    );
}

#[test]
fn a_record_field_is_snake_case() {
    assert_eq!(
        help("type User = {\n    userName: String\n}\n"),
        "canonical form spells this name `user_name`"
    );
}

#[test]
fn a_variant_is_pascal_case() {
    assert_eq!(
        help("type Payment =\n    | pending\n    | Failed(String)\n"),
        "canonical form spells this name `Pending`"
    );
}

#[test]
fn a_field_of_a_record_variant_is_snake_case() {
    assert_eq!(
        help(
            "type Payment =\n    | Pending\n    | Authorized {\n        authorizationID: String\n    }\n"
        ),
        "canonical form spells this name `authorization_id`"
    );
}

#[test]
fn a_name_that_leads_or_doubles_an_underscore_is_spelled_without_it() {
    assert_eq!(
        help("fn __the_word_(word: String) -> String {\n    word\n}\n"),
        "canonical form spells this name `the_word`"
    );
}

#[test]
fn the_message_says_what_the_name_names_and_the_case_that_kind_is_written_in() {
    assert_eq!(
        message("type UserID = UserID(Int)\n"),
        "error[L0202]: `UserID` is a type, so canonical form writes it in `PascalCase`"
    );
}

#[test]
fn a_name_of_one_character_names_nothing_a_reader_can_look_for() {
    assert_eq!(
        message("fn f(count: Int) -> Int {\n    count\n}\n"),
        "error[L0203]: `f` is an initial, which names nothing a reader can look for"
    );
}

#[test]
fn a_name_of_two_characters_is_a_word_and_compiles() {
    assert!(check("fn go(count: Int) -> Int {\n    count\n}\n").is_ok());
}

#[test]
fn a_type_parameter_is_exempt_because_it_names_no_domain_concept() {
    assert!(check("fn identity<T>(value: T) -> T {\n    value\n}\n").is_ok());
}

#[test]
fn a_name_that_is_both_an_initial_and_miscased_is_refused_as_the_initial() {
    assert_eq!(
        message("type F = F(Int)\n"),
        "error[L0203]: `F` is an initial, which names nothing a reader can look for"
    );
}

#[test]
fn a_local_binding_is_private_to_its_body_so_no_rule_here_is_about_one() {
    assert!(check("fn go(count: Int) -> Int {\n    aBc := count\n    aBc\n}\n").is_ok());
}

#[test]
fn an_imported_module_is_snake_case() {
    assert_eq!(
        help("import myIo\n"),
        "canonical form spells this name `my_io`"
    );
}

#[test]
fn the_first_name_a_file_writes_out_of_form_is_the_one_reported() {
    assert_eq!(
        message("fn go(theWord: String, alsoWrong: String) -> String {\n    theWord\n}\n"),
        "error[L0202]: `theWord` is a parameter, so canonical form writes it in `snake_case`"
    );
}

#[test]
fn a_refusal_reads_as_the_diagnostic_it_is() {
    assert_eq!(
        refusal("type UserID = UserID(Int)\n"),
        concat!(
            "error[L0202]: `UserID` is a type, so canonical form writes it in `PascalCase`\n",
            "  --> demo.lm:1:6\n",
            "\n",
            "  1 | type UserID = UserID(Int)\n",
            "    |      ^^^^^^\n",
            "\n",
            "help: canonical form spells this name `UserId`\n",
        )
    );
}

#[test]
fn a_name_whose_word_is_one_letter_is_an_initial_however_it_is_punctuated() {
    assert_eq!(
        message("fn _a(count: Int) -> Int {\n    count\n}\n"),
        "error[L0203]: `_a` is an initial, which names nothing a reader can look for"
    );
}

#[test]
fn a_name_that_is_nothing_but_underscores_names_no_word_at_all() {
    assert_eq!(
        message("fn __(count: Int) -> Int {\n    count\n}\n"),
        "error[L0203]: `__` is an initial, which names nothing a reader can look for"
    );
}

//! What an `extern` declaration is held to, which `docs/specs/interop.md` states.
//!
//! The boundary is a signature and nothing more, so every refusal here is one about a signature.
//! A member the JVM does not have is not among them: nothing is loaded to refuse it against.

use crate::common::{inferred_type, refusal};

/// A module declaring `written`, over the two extern types a declaration here may name.
fn declaring(written: &str) -> String {
    format!(
        "extern {written}\n\nextern type File = \"java.io.File\"\n\nextern type Path = \"java.nio.file.Path\"\n"
    )
}

#[test]
fn a_type_no_java_member_takes_is_refused_where_the_signature_writes_it() {
    let declared = declaring("static written(user: User) -> String = \"java.lang.String.valueOf\"");
    let error = refusal(&format!(
        "{declared}\ntype User = {{\n    name: String\n}}\n"
    ));

    assert_eq!(error.message(), "`User` is no type a Java member takes");
    assert_eq!(
        error.help(),
        "a boundary carries `Bool`, `Int`, `String`, a `List`, and a type an `extern` names"
    );
}

#[test]
fn a_list_is_taken_as_the_java_util_list_a_jvm_already_holds_it_as() {
    let source = "fn built() -> ProcessBuilder {\n    of_command([\"java\"])\n}\n\nextern new of_command(command: List<String>) -> ProcessBuilder\n\nextern type ProcessBuilder = \"java.lang.ProcessBuilder\"\n";

    assert_eq!(
        inferred_type(source, "of_command([\"java\"])", 1),
        "ProcessBuilder"
    );
}

#[test]
fn a_list_a_member_gives_back_is_refused_because_that_member_may_still_change_it() {
    let error = refusal(&declaring(
        "static written(path: Path) -> List<String> = \"java.nio.file.Files.readAllLines\"",
    ));

    assert_eq!(
        error.message(),
        "`List<String>` is no type a Java member gives back"
    );
}

#[test]
fn an_option_and_a_result_are_a_result_and_never_what_a_member_takes() {
    let error = refusal(&declaring(
        "static written(held: Option<File>) -> String = \"java.lang.String.valueOf\"",
    ));

    assert_eq!(
        error.message(),
        "`Option<File>` is no type a Java member takes"
    );
}

#[test]
fn neither_answer_wraps_a_unit_because_neither_has_anything_to_carry() {
    for written in [
        "method held(file: File) -> Result<(), String> = \"deleteOnExit\"",
        "method held(file: File) -> Option<()> = \"deleteOnExit\"",
    ] {
        let message = refusal(&declaring(written)).message();
        assert!(
            message.ends_with("is no type a Java member gives back"),
            "{message}"
        );
    }
}

#[test]
fn a_field_holding_nothing_at_all_is_no_field_because_the_jvm_has_no_void_one() {
    let error = refusal(&declaring("field held() -> () = \"java.lang.System.out\""));

    assert_eq!(error.message(), "`()` is no type a Java member holds");
    assert_eq!(
        error.help(),
        "a boundary carries `Bool`, `Int`, `String`, a `List`, and a type an `extern` names"
    );
}

#[test]
fn a_field_still_gives_back_an_answer_because_reading_one_can_fail_or_be_null() {
    for answer in ["Option<File>", "Result<File, String>"] {
        let written = format!("field held() -> {answer} = \"java.io.File.listRoots\"");
        let source = format!(
            "fn read() -> {answer} {{\n    held()\n}}\n\n{}",
            declaring(&written)
        );

        assert_eq!(inferred_type(&source, "held()", 1), answer);
    }
}

#[test]
fn a_java_name_with_an_empty_segment_names_no_class_the_jvm_could_hold() {
    let error = refusal("extern type File = \"java..File\"\n");

    assert_eq!(error.message(), "`java..File` is no Java name");
    assert_eq!(
        error.help(),
        "a Java name is its segments, each a name, with a dot between two of them"
    );
}

#[test]
fn a_field_and_a_static_name_the_class_as_well_as_the_member() {
    let field = refusal(&declaring("field out() -> File = \"out\""));
    let called = refusal(&declaring(
        "static held(path: Path) -> String = \"readString\"",
    ));

    assert_eq!(field.message(), "`out` is no Java name");
    assert_eq!(called.message(), "`readString` is no Java name");
}

#[test]
fn a_method_names_the_member_alone_because_its_receiver_says_which_class() {
    let error = refusal(&declaring(
        "method held(file: File) -> Path = \"java.io.File.toPath\"",
    ));

    assert_eq!(error.message(), "`java.io.File.toPath` is no Java name");
}

#[test]
fn a_derive_of_an_extern_type_is_refused_because_what_it_holds_is_the_jvm_s() {
    let error = refusal("derive Eq for File\n\nextern type File = \"java.io.File\"\n");

    assert_eq!(
        error.message(),
        "`File` is an extern type, and a derive reads what a type holds"
    );
    assert_eq!(
        error.help(),
        "write an `instance` over `extern` declarations instead"
    );
}

#[test]
fn a_method_whose_first_parameter_names_no_class_reaches_nothing() {
    let error = refusal(&declaring(
        "method said(count: Int) -> String = \"toString\"",
    ));

    assert_eq!(
        error.message(),
        "a `method` reaches a class, and this signature names none"
    );
    assert_eq!(
        error.help(),
        "a class is `String`, or a type an `extern type` declares"
    );
}

#[test]
fn a_new_whose_result_names_no_class_builds_nothing() {
    let error = refusal(&declaring("new counted(path: String) -> Int"));

    assert_eq!(
        error.message(),
        "a `new` reaches a class, and this signature names none"
    );
}

#[test]
fn an_extern_is_an_ordinary_function_of_the_module_that_declares_it() {
    let source = concat!(
        "fn held(file: File) -> Path {\n    as_a_path(file)\n}\n\n",
        "extern method as_a_path(file: File) -> Path = \"toPath\"\n\n",
        "extern type File = \"java.io.File\"\n\n",
        "extern type Path = \"java.nio.file.Path\"\n",
    );

    assert_eq!(inferred_type(source, "as_a_path(file)", 1), "Path");
}

#[test]
fn a_width_written_where_the_member_gives_back_no_int_has_nothing_to_widen_to() {
    let error = refusal(&declaring(
        "method int as_a_path(file: File) -> Path = \"toPath\"",
    ));

    assert_eq!(
        error.message(),
        "`int` widens to an `Int`, and this signature gives back `Path`"
    );
    assert_eq!(
        error.help(),
        "a width widens to an `Int`; drop the word, or give an `Int` back"
    );
}

#[test]
fn a_new_writing_the_width_meets_that_one_rule_the_way_every_other_kind_does() {
    let error = refusal(&declaring("new int opened(path: String) -> File"));

    assert_eq!(
        error.message(),
        "`int` widens to an `Int`, and this signature gives back `File`"
    );
}

#[test]
fn a_width_is_accepted_through_the_result_a_declaration_wraps_it_in() {
    let source = "fn held(text: String) -> Result<Int, String> {\n    counted(text)\n}\n\n"
        .to_owned()
        + &declaring(
            "static int counted(text: String) -> Result<Int, String> = \"java.lang.String.length\"",
        );

    assert_eq!(
        inferred_type(&source, "counted(text)", 1),
        "Result<Int, String>"
    );
}

#[test]
fn a_char_is_held_to_the_same_result_the_other_width_is() {
    let error = refusal(&declaring(
        "method char as_a_path(file: File) -> Path = \"toPath\"",
    ));

    assert_eq!(
        error.message(),
        "`char` widens to an `Int`, and this signature gives back `Path`"
    );
    assert_eq!(
        error.help(),
        "a width widens to an `Int`; drop the word, or give an `Int` back",
        "the help names no word the author did not write"
    );
}

#[test]
fn a_narrowed_parameter_takes_an_int_and_nothing_else() {
    let error = refusal(&declaring(
        "method int at(text: String, int held: String) -> Option<Int> = \"charAt\"",
    ));

    assert_eq!(
        error.message(),
        "`int` narrows an `Int`, and this parameter takes `String`"
    );
    assert_eq!(
        error.help(),
        "an `int` narrows an `Int`; drop the word, or take an `Int`"
    );
}

#[test]
fn a_declaration_that_narrows_an_argument_gives_back_an_option_to_say_none_with() {
    let error = refusal(&declaring(
        "method char at(text: String, int index: Int) -> Int = \"charAt\"",
    ));

    assert_eq!(
        error.message(),
        "an `int` parameter narrows an `Int`, and this signature gives back `Int`"
    );
    assert_eq!(
        error.help(),
        "give back an `Option`, which is `None` where an argument does not fit"
    );
}

#[test]
fn an_option_over_a_number_is_the_result_of_a_declaration_that_narrows_an_argument() {
    let source = "fn held(text: String) -> Option<Int> {\n    at(text, 0)\n}\n\n".to_owned()
        + &declaring("method char at(text: String, int index: Int) -> Option<Int> = \"charAt\"");

    assert_eq!(inferred_type(&source, "at(text, 0)", 1), "Option<Int>");
}

#[test]
fn an_option_over_a_number_is_no_result_where_nothing_is_narrowed() {
    let error = refusal(&declaring(
        "method int at(text: String, index: Int) -> Option<Int> = \"charAt\"",
    ));

    assert_eq!(
        error.message(),
        "`Option<Int>` is no type a Java member gives back"
    );
}

#[test]
fn a_guard_stands_outside_the_option_a_narrowed_argument_answers_with() {
    let written = "method cut(text: String, int from: Int) -> Result<Option<String>, String>";
    let source =
        "fn held(text: String) -> Result<Option<String>, String> {\n    cut(text, 0)\n}\n\n"
            .to_owned()
            + &declaring(&format!("{written} = \"substring\""));

    assert_eq!(
        inferred_type(&source, "cut(text, 0)", 1),
        "Result<Option<String>, String>"
    );
}

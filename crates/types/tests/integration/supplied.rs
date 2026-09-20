//! The modules the compiler declares, as `docs/specs/io.md` states them.
//!
//! Inference is given the surface of every module loading read, and these are inferred with
//! none, so the only modules reached here are the two the compiler supplies.

use crate::common::{inferred, refusal};

/// A module importing `named` and calling `written` with one string.
fn calling(named: &str, written: &str) -> String {
    format!("import {named}\n\nfn main() -> () {{\n    {written}(\"text\")\n}}\n")
}

#[test]
fn io_writes_out_a_string_and_gives_nothing_back_under_either_name() {
    inferred(&calling("io", "io.print"));
    inferred(&calling("io", "io.println"));
}

#[test]
fn writing_a_line_out_leaves_nothing_behind_so_one_call_follows_another() {
    let source =
        "import io\n\nfn main() -> () {\n    io.println(\"one\")\n    io.println(\"two\")\n}\n";

    inferred(source);
}

#[test]
fn what_is_written_out_is_a_string_and_nothing_else() {
    assert_eq!(
        refusal("import io\n\nfn main() -> () {\n    io.println(7)\n}\n").message(),
        "expected `String`, found `Int`"
    );
}

#[test]
fn reading_a_file_gives_back_a_result_the_caller_must_open() {
    let source = "import files\n\nfn read(path: String) -> Result<String, String> {\n    files.read(path)\n}\n";

    inferred(source);
}

#[test]
fn what_a_file_read_gives_back_is_not_the_text_itself() {
    let source = "import files\n\nfn read(path: String) -> String {\n    files.read(path)\n}\n";

    assert_eq!(
        refusal(source).message(),
        "expected `String`, found `Result<String, String>`"
    );
}

#[test]
fn a_name_a_supplied_module_does_not_declare_is_refused_where_it_is_written() {
    let error = refusal(&calling("io", "io.write"));

    assert_eq!(error.message(), "`io` declares no `write`");
    assert_eq!(
        error.help(),
        "a module declares the functions it offers, and nothing else is a name it has"
    );
}

#[test]
fn a_supplied_module_declares_its_own_names_and_none_of_another_module_s() {
    assert_eq!(
        refusal(&calling("files", "files.println")).message(),
        "`files` declares no `println`"
    );
    assert_eq!(
        refusal(&calling("io", "io.read")).message(),
        "`io` declares no `read`"
    );
}

#[test]
fn a_module_whose_surface_inference_was_not_given_declares_nothing() {
    assert_eq!(
        refusal(&calling("other", "other.write")).message(),
        "`other` declares no `write`"
    );
}

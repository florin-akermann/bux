//! The two library modules that reach outside a program, as `docs/specs/io.md` states them.
//!
//! `io` and `files` are Lumen source the compiler carries, so inference is given their surfaces
//! the way it is given any loaded module's. What those surfaces say is what these hold to.

use crate::common::{inferred_reaching, reaching_the_library, refusal_reaching};

/// A module importing `named` and calling `written` with one string.
fn calling(named: &str, written: &str) -> String {
    format!("import {named}\n\nfn main() -> () {{\n    {written}(\"text\")\n}}\n")
}

#[test]
fn io_writes_out_a_string_and_gives_nothing_back_under_either_name() {
    let library = reaching_the_library();

    inferred_reaching(&calling("io", "io.print"), &library);
    inferred_reaching(&calling("io", "io.println"), &library);
}

#[test]
fn writing_a_line_out_leaves_nothing_behind_so_one_call_follows_another() {
    let source =
        "import io\n\nfn main() -> () {\n    io.println(\"one\")\n    io.println(\"two\")\n}\n";

    inferred_reaching(source, &reaching_the_library());
}

#[test]
fn what_is_written_out_is_a_string_and_nothing_else() {
    let source = "import io\n\nfn main() -> () {\n    io.println(7)\n}\n";

    assert_eq!(
        refusal_reaching(source, &reaching_the_library()).message(),
        "expected `String`, found `Int`"
    );
}

#[test]
fn reading_a_file_gives_back_a_result_the_caller_must_open() {
    let source = "import files\n\nfn read(path: String) -> Result<String, String> {\n    files.read(path)\n}\n";

    inferred_reaching(source, &reaching_the_library());
}

#[test]
fn what_a_file_read_gives_back_is_not_the_text_itself() {
    let source = "import files\n\nfn read(path: String) -> String {\n    files.read(path)\n}\n";

    assert_eq!(
        refusal_reaching(source, &reaching_the_library()).message(),
        "expected `String`, found `Result<String, String>`"
    );
}

#[test]
fn a_name_a_library_module_does_not_declare_is_refused_where_it_is_written() {
    let error = refusal_reaching(&calling("io", "io.write"), &reaching_the_library());

    assert_eq!(error.message(), "`io` declares no `write`");
    assert_eq!(
        error.help(),
        "a module offers the functions and the types it declares, and nothing else"
    );
}

#[test]
fn a_library_module_declares_its_own_names_and_none_of_another_module_s() {
    let library = reaching_the_library();

    assert_eq!(
        refusal_reaching(&calling("files", "files.println"), &library).message(),
        "`files` declares no `println`"
    );
    assert_eq!(
        refusal_reaching(&calling("io", "io.read"), &library).message(),
        "`io` declares no `read`"
    );
}

#[test]
fn a_module_whose_surface_inference_was_not_given_declares_nothing() {
    let source = calling("other", "other.write");

    assert_eq!(
        refusal_reaching(&source, &reaching_the_library()).message(),
        "`other` declares no `write`"
    );
}

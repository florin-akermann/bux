//! The library the compiler carries, held to what every module is held to.
//!
//! `docs/specs/library.md` states the properties these check: the library is ordinary Lumen
//! source, so it is in canonical form and it compiles, and a program reaches a module of it by
//! importing it and by nothing else.

use lumen_format::format;
use lumen_resolver::library;

use crate::common::{Example, jdk, lumen};

#[test]
fn every_library_module_is_in_canonical_form() {
    for (name, source) in library::carried() {
        let canonical = format(source).unwrap_or_else(|error| {
            panic!("library/{name}.lm parses: {}", error.diagnostic().message())
        });

        assert_eq!(
            canonical, source,
            "library/{name}.lm is not in canonical form"
        );
    }
}

#[test]
fn every_library_module_a_program_may_import_is_accepted_as_a_module_of_its_own() {
    for (name, _) in library::carried() {
        let Some(source) = library::imported(name) else {
            continue;
        };
        let example = Example::new(source);

        let run = lumen(&["check", example.path.to_str().expect("a UTF-8 path")]);

        assert_eq!(run.code, 0, "library/{name}.lm: {}", run.stderr);
    }
}

#[test]
fn every_example_a_library_module_states_holds() {
    if jdk().is_none() {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the examples needs one");
        return;
    }
    for (name, _) in library::carried() {
        let Some(source) = library::imported(name) else {
            continue;
        };
        let example = Example::new(source);

        let run = lumen(&["test", example.path.to_str().expect("a UTF-8 path")]);

        assert_eq!(run.code, 0, "library/{name}.lm: {}", run.stderr);
    }
}

#[test]
fn a_program_that_imports_no_library_module_reaches_none_of_their_names() {
    let example = Example::new(
        "fn main(arguments: List<String>) -> Int {\n    _ = strings.join([], \"-\")\n    0\n}\n",
    );

    let run = lumen(&["check", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 1);
    assert!(run.stderr.starts_with("error[L0300]: "), "{}", run.stderr);
}

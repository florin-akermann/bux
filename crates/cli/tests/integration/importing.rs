//! A program written across several modules, as the command line compiles and runs it.
//!
//! `docs/specs/modules.md` states where an import finds its file and which order the modules it
//! reaches are compiled in. Every command reads the same modules, so one is checked, built, run,
//! and tested as a whole.

use crate::common::{Example, Sibling, jdk, lumen};

/// The module a program imports, offering one function written in types both modules have.
const GREETING: &str = "// example: greeting(name: \"world\") == \"hello, world\"\nfn greeting(name: String) -> String {\n    \"hello, \" + name\n}\n";

/// A program reaching that function, which is the module every test here names.
const PROGRAM: &str = "import greeting\n\nimport io\n\nfn main() -> () {\n    io.println(greeting.greeting(\"world\"))\n}\n";

/// An example of `PROGRAM` with `GREETING` written beside it under the name it is imported by.
fn program() -> Example {
    let example = Example::new(PROGRAM);
    example.beside_it(&Sibling {
        named: "greeting",
        content: GREETING,
    });
    example
}

/// The path of `example`, as an argument is written.
fn named(example: &Example) -> &str {
    example.path.to_str().expect("a UTF-8 path")
}

#[test]
fn check_accepts_a_program_whose_import_names_a_file_beside_it() {
    let example = program();

    let run = lumen(&["check", named(&example)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn check_refuses_a_program_whose_import_names_no_file_beside_it() {
    let example = Example::new(PROGRAM);

    let run = lumen(&["check", named(&example)]);

    assert_eq!(run.code, 1);
    assert!(run.stderr.starts_with("error[L0306]: "), "{}", run.stderr);
}

#[test]
fn a_refusal_of_an_imported_module_names_that_module_s_own_file() {
    let example = Example::new(PROGRAM);
    example.beside_it(&Sibling {
        named: "greeting",
        content: "fn greeting(name: String) -> Int {\n    name\n}\n",
    });

    let run = lumen(&["check", named(&example)]);

    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("greeting.lm"), "{}", run.stderr);
}

#[test]
fn build_writes_a_class_for_every_module_the_program_reaches() {
    let example = program();

    let run = lumen(&["build", named(&example)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(example.beside("example.class").is_some(), "the program");
    assert!(
        example.beside("greeting.class").is_some(),
        "what it imports"
    );
}

#[test]
fn build_writes_nothing_for_a_module_beside_it_that_nothing_imports() {
    let example = program();
    example.beside_it(&Sibling {
        named: "unreached",
        content: "fn held() -> Int {\n    1\n}\n",
    });

    let run = lumen(&["build", named(&example)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(example.beside("unreached.class").is_none());
}

#[test]
fn run_starts_the_program_and_what_it_reaches_writes_through_it() {
    let example = program();
    if jdk().is_none() {
        eprintln!("skipped: JAVA_HOME names no JDK, and running a program needs one");
        return;
    }

    let run = lumen(&["run", named(&example)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "hello, world\n");
}

#[test]
fn test_runs_the_examples_of_a_module_that_imports_another() {
    let source = "import greeting\n\n// example: twice() == \"hello, world\"\nfn twice() -> String {\n    greeting.greeting(\"world\")\n}\n";
    let example = Example::new(source);
    example.beside_it(&Sibling {
        named: "greeting",
        content: GREETING,
    });
    if jdk().is_none() {
        eprintln!("skipped: JAVA_HOME names no JDK, and running an example needs one");
        return;
    }

    let run = lumen(&["test", named(&example)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn api_prints_the_surface_of_the_module_named_and_not_of_what_it_imports() {
    let example = program();

    let run = lumen(&["api", named(&example)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.contains("fn main"), "{}", run.stdout);
    assert!(!run.stdout.contains("fn greeting"), "{}", run.stdout);
}

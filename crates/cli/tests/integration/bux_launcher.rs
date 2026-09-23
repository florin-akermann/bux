//! The launcher `bin/bux`, which starts the command line written in Bux on a JVM.
//!
//! `docs/specs/run.md` states what it does and what it refuses, and
//! `docs/specs/executable-examples.md` states that every example holds under it. Its refusals need
//! no JDK; a run needs one, and a test that runs one is skipped with a named reason without it.

use std::fs;

use crate::common::{Case, Home, LAUNCHED, Stage, fixture_cases, jdk, the_same_from_the_launcher};

#[test]
fn the_bux_command_line_builds_under_the_rust_compiler() {
    let stage = Stage::built(LAUNCHED);

    assert!(stage.root.join("compiler/main.class").is_file());
}

/// `docs/specs/executable-examples.md`: every program under `tests/spec` runs under the launcher
/// as it runs under `lumen`.
#[test]
fn every_program_under_tests_spec_runs_under_the_launcher_as_under_lumen() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running a program needs one");
        return;
    };
    let stage = Stage::built(LAUNCHED);

    the_same_from_the_launcher(&stage, &fixture_cases(&["run"], &["tests/spec"]));
}

/// `docs/specs/run.md`: the launcher refuses to start a compiler that is not built.
#[test]
fn the_launcher_refuses_a_compiler_that_is_not_built() {
    let stage = Stage::staged();

    let said = stage.said_by_the_launcher(&Case::written("--version"), 0);

    assert_eq!(said.status, 2);
    assert!(
        said.errors
            .ends_with("the compiler is not built; bin/bootstrap builds it\n")
    );
}

/// `docs/specs/run.md`: the launcher names the variable a reader sets when no JDK is named.
#[test]
fn the_launcher_refuses_to_start_without_java_home() {
    let stage = with_a_class_to_start();

    let said = stage.said_by_the_launcher_finding(&Case::written("--version"), 0, Home::Unset);

    assert_eq!(said.status, 2);
    assert_eq!(
        said.errors,
        "error: JAVA_HOME is not set, and the bux compiler runs on the JDK it names\n"
    );
}

/// `docs/specs/run.md`: the launcher refuses a `JAVA_HOME` that holds no `bin/java`.
#[test]
fn the_launcher_refuses_a_java_home_that_holds_no_jdk() {
    let stage = with_a_class_to_start();

    let said = stage.said_by_the_launcher_finding(
        &Case::written("--version"),
        0,
        Home::At("/no/such/jdk"),
    );

    assert_eq!(said.status, 2);
    assert_eq!(
        said.errors,
        "error: /no/such/jdk/bin/java: JAVA_HOME names no JDK\n"
    );
}

/// A stage whose launcher finds a class to start, which is all it looks for before the JDK.
///
/// The launcher never starts the class here, so an empty file stands in for the compiler.
fn with_a_class_to_start() -> Stage {
    let stage = Stage::staged();
    fs::write(stage.root.join("compiler/main.class"), b"").expect("a class file is writable");
    stage
}

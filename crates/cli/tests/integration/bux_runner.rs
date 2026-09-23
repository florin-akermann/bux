//! The runner under `tests/`, which holds the Bux compiler to every example, every example line,
//! and every drawn property, with no Rust.
//!
//! `docs/implementation.md` section 7 states the runner. This test proves that it holds before the
//! Rust crates go: `lumen` builds the compiler on a stage, `bin/runner` runs there, and nothing
//! fails. The runner runs on a JVM, so without a JDK the test is skipped with a named reason.

use std::process::{Command, Stdio};

use crate::common::{LAUNCHED, Stage, jdk};

/// `docs/implementation.md` section 7: the runner ends with status 0 only when nothing failed.
#[test]
fn the_runner_holds_the_bux_compiler_to_everything_under_tests() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and the runner runs on one");
        return;
    };
    let stage = Stage::built(LAUNCHED);

    let ran = Command::new(stage.root.join("bin/runner"))
        .current_dir(&stage.root)
        .stdin(Stdio::null())
        .output()
        .expect("the runner starts");

    let output = String::from_utf8_lossy(&ran.stdout);
    let errors = String::from_utf8_lossy(&ran.stderr);
    assert!(ran.status.success(), "{output}{errors}");
    assert!(output.contains("; 0 failed"), "{output}");
}

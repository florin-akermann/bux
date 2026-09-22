//! The one program in the tree somebody would actually write.
//!
//! `docs/specs/example-program.md` says what `example/main.lm` is and what running it does.

use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use lumen_format::format;

use crate::common::{Example, jdk, lumen};

/// `example/main.lm`, which the README points a newcomer at.
fn program() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../example/main.lm")
        .canonicalize()
        .expect("example/main.lm is in the tree")
}

fn source() -> String {
    read_to_string(program()).expect("example/main.lm is readable")
}

#[test]
fn the_example_program_compiles_like_any_other_module() {
    let run = lumen(&["check", program().to_str().expect("a UTF-8 path")]);

    assert_eq!(
        run.code, 0,
        "example/main.lm does not compile:\n{}",
        run.stderr
    );
}

#[test]
fn the_example_program_is_in_canonical_form() {
    let source = source();

    assert_eq!(
        format(&source).expect("example/main.lm parses"),
        source,
        "example/main.lm is not in canonical form"
    );
}

/// What a run of the example program writes, which `docs/specs/example-program.md` states.
const WRITTEN: &str = "held: 7\neach: 3\nmost: 6\n";

#[test]
fn the_example_program_writes_every_answer_it_works_out() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the example program needs one");
        return;
    };
    let copy = Example::new(&source());

    let run = lumen(&["run", copy.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(
        run.code, 0,
        "example/main.lm does not run to the end:\n{}",
        run.stderr
    );
    assert_eq!(run.stdout, WRITTEN, "example/main.lm writes another answer");
}

#[test]
fn the_examples_of_the_example_program_hold() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the examples needs one");
        return;
    };
    let copy = Example::new(&source());

    let run = lumen(&["test", copy.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(
        run.code, 0,
        "an example of example/main.lm does not hold:\n{}",
        run.stderr
    );
}

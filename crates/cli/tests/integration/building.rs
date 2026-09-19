//! `lumen build`: the class files a module becomes, as the command line writes them.

use std::path::Path;
use std::process::Command;

use crate::common::{Example, jdk, lumen};

/// A module with a record, an algebraic data type, and a function over each.
const MODULE: &str = "fn told(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Failed(reason) => reason\n    }\n}\n\ntype Payment =\n    | Pending\n    | Failed(String)\n\ntype User = {\n    id: Int\n}\n";

#[test]
fn build_writes_one_class_beside_the_source_for_the_module_and_each_type() {
    let example = Example::new(MODULE);

    let run = lumen(&["build", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    for written in [
        "example.class",
        "example/User.class",
        "example/Payment.class",
        "example/Payment$Pending.class",
        "example/Payment$Failed.class",
        "lumen/Option.class",
        "lumen/Result$Ok.class",
    ] {
        assert!(example.beside(written).is_some(), "{written} is written");
    }
}

#[test]
fn every_class_build_writes_begins_with_the_class_file_magic() {
    let example = Example::new(MODULE);

    let run = lumen(&["build", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    let bytes = example
        .beside("example.class")
        .expect("the module is written");
    assert_eq!(&bytes[..4], &[0xCA, 0xFE, 0xBA, 0xBE]);
    assert_eq!(
        &bytes[4..8],
        &[0xFF, 0xFF, 0, 72],
        "JDK 28's version, marked preview"
    );
}

#[test]
fn build_writes_the_same_bytes_every_time_it_is_run() {
    let example = Example::new(MODULE);

    lumen(&["build", example.path.to_str().expect("a UTF-8 path")]);
    let first = example
        .beside("example.class")
        .expect("the module is written");
    lumen(&["build", example.path.to_str().expect("a UTF-8 path")]);
    let again = example
        .beside("example.class")
        .expect("the module is written");

    assert_eq!(first, again, "nothing about a build depends on when it ran");
}

#[test]
fn build_writes_nothing_when_the_compiler_refuses_the_program() {
    let example = Example::new("fn told(payment: Int) -> String {\n    payment\n}\n");

    let run = lumen(&["build", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("error[L0400]"), "{}", run.stderr);
    assert!(example.beside("example.class").is_none());
}

#[test]
fn build_reports_a_file_it_cannot_read_without_saying_anything_about_a_program() {
    let run = lumen(&["build", "no-such-file.lm"]);

    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("no-such-file.lm"), "{}", run.stderr);
}

/// A module reaching every construct whose lowering the verifier has something to say about.
const EVERYTHING: &str = "fn walked(counts: List<Int>) -> Int {\n    var total = 0\n    for count in counts {\n        total += identity(count)\n    }\n    total\n}\n\nfn joined(word: String) -> String {\n    identity(word) + \"!\"\n}\n\nfn compared(count: Int) -> Bool {\n    identity(count) < 2\n}\n\nfn negated(flag: Bool) -> Bool {\n    !identity(flag)\n}\n\nfn picked(count: Int) -> Int {\n    if identity(count > 1) {\n        1\n    } else {\n        2\n    }\n}\n\nfn identity<T>(value: T) -> T {\n    value\n}\n";

#[test]
fn every_class_build_writes_is_one_a_jvm_loads_and_verifies() {
    let Some(java) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and only a JVM verifies a class file");
        return;
    };
    let example = Example::new(EVERYTHING);
    let run = lumen(&["build", example.path.to_str().expect("a UTF-8 path")]);
    assert_eq!(run.code, 0, "{}", run.stderr);

    let written = classes_under(&example.directory, "");
    assert!(!written.is_empty(), "the build wrote something to load");
    for class in written {
        let output = Command::new(&java)
            .args(["--enable-preview", "-cp", directory(&example), &class])
            .output()
            .expect("the JDK's java runs");
        let said = String::from_utf8_lossy(&output.stderr);
        assert!(
            !said.contains("Error") || said.contains("Main method not found"),
            "{class} did not load: {said}"
        );
    }
}

/// Every class written under `directory`, named the way a JVM is asked for one.
fn classes_under(directory: &Path, package: &str) -> Vec<String> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.path().is_dir() {
            found.extend(classes_under(&entry.path(), &format!("{package}{name}.")));
        } else if let Some(class) = name.strip_suffix(".class") {
            found.push(format!("{package}{class}"));
        }
    }
    found
}

fn directory(example: &Example) -> &str {
    example
        .directory
        .to_str()
        .expect("a UTF-8 temporary directory")
}

#[test]
fn build_refuses_a_file_whose_name_is_not_one_a_class_may_have() {
    let example = Example::new(MODULE);
    let dotted = example.directory.join("my.app.lm");
    std::fs::copy(&example.path, &dotted).expect("the example is copyable");

    let run = lumen(&["build", dotted.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(run.stderr.contains("my.app.lm"), "{}", run.stderr);
    assert!(example.beside("my.app.class").is_none());
}

/// `tests/spec/holes/unfinished.lm`, which the example harness holds `lumen check` to.
const UNFINISHED: &str = concat!(
    "fn total(users: List<User>) -> Int {\n    todo(\"count them once the walk is written\")\n}\n\n",
    "fn named(user: User) -> String {\n    todo(\"read the name off the record\")\n}\n\n",
    "type User = {\n    id: Int\n}\n"
);

#[test]
fn build_refuses_a_module_that_holds_a_hole() {
    let example = Example::new(UNFINISHED);

    let run = lumen(&["build", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(
        run.stderr
            .starts_with("error[L0600]: this hole is not compiled\n"),
        "{}",
        run.stderr
    );
}

#[test]
fn build_names_every_hole_rather_than_the_first() {
    let example = Example::new(UNFINISHED);

    let run = lumen(&["build", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(
        run.stderr.matches("error[L0600]").count(),
        2,
        "a build is how a reader learns what is left:\n{}",
        run.stderr
    );
}

#[test]
fn a_blank_line_keeps_two_refused_holes_two_blocks() {
    let example = Example::new(UNFINISHED);

    let run = lumen(&["build", example.path.to_str().expect("a UTF-8 path")]);

    assert!(
        run.stderr.contains("holds none\n\nerror[L0600]"),
        "one block ends and the next begins:\n{}",
        run.stderr
    );
}

#[test]
fn build_writes_nothing_when_a_module_holds_a_hole() {
    let example = Example::new(UNFINISHED);

    lumen(&["build", example.path.to_str().expect("a UTF-8 path")]);

    assert!(
        example.beside("example.class").is_none(),
        "a hole has nothing to run, so there is nothing to write"
    );
}

#[test]
fn check_accepts_the_very_module_build_refuses() {
    let example = Example::new(UNFINISHED);

    let run = lumen(&["check", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

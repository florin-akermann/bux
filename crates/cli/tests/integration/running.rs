//! `lumen run`: a source file compiled and then run, which `docs/specs/run.md` states.

use std::path::Path;

use crate::common::{Example, jdk, lumen, lumen_finding};

/// A program: a module declaring the `main` a run starts at, over a type and a loop.
const PROGRAM: &str = "fn main() -> () {\n    ()\n}\n\n// example: counted([User { id: 1, active: true }]) == 1\nfn counted(users: List<User>) -> Int {\n    var total = 0\n    for user in users {\n        if user.active {\n            total += 1\n        }\n    }\n    total\n}\n\ntype User = {\n    id: Int\n    active: Bool\n}\n";

/// A library: a module the compiler accepts, and that a run has nothing to start.
const LIBRARY: &str = "// example: counted() == 2\nfn counted() -> Int {\n    2\n}\n";

#[test]
fn run_compiles_a_program_and_runs_it_to_the_end() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running a program needs one");
        return;
    };
    let example = Example::new(PROGRAM);

    let run = lumen(&["run", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "", "a program with nothing to say says nothing");
}

#[test]
fn run_leaves_behind_exactly_what_building_the_same_file_leaves() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running a program needs one");
        return;
    };
    let built = Example::new(PROGRAM);
    let started = Example::new(PROGRAM);

    lumen(&["build", built.path.to_str().expect("a UTF-8 path")]);
    lumen(&["run", started.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(
        written_by(&built),
        written_by(&started),
        "a run builds, and builds nothing else"
    );
}

#[test]
fn run_refuses_a_module_that_declares_no_main() {
    let example = Example::new(LIBRARY);

    let run = lumen(&["run", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(run.stderr.contains("does not declare"), "{}", run.stderr);
    assert!(
        run.stderr.contains("fn main() -> ()"),
        "it names the shape to write: {}",
        run.stderr
    );
}

#[test]
fn run_refuses_a_program_the_compiler_will_not_have_and_runs_nothing() {
    let example = Example::new("fn main() -> () {\n    missing()\n}\n");

    let run = lumen(&["run", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 1, "{}", run.stderr);
    assert!(run.stderr.starts_with("error[L03"), "{}", run.stderr);
    assert!(example.beside("example.class").is_none());
}

#[test]
fn run_says_which_variable_names_the_jdk_when_nothing_does() {
    let example = Example::new(PROGRAM);

    let run = lumen_finding(&["run", example.path.to_str().expect("a UTF-8 path")], None);

    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(
        run.stderr.contains("JAVA_HOME is not set"),
        "{}",
        run.stderr
    );
    assert!(
        example.beside("example.class").is_some(),
        "the program was compiled before a JVM was looked for"
    );
}

#[test]
fn run_reports_a_program_that_does_not_compile_even_where_no_jdk_could_run_it() {
    let example = Example::new("fn main() -> () {\n    missing()\n}\n");

    let run = lumen_finding(&["run", example.path.to_str().expect("a UTF-8 path")], None);

    assert_eq!(run.code, 1, "{}", run.stderr);
    assert!(run.stderr.starts_with("error[L03"), "{}", run.stderr);
}

#[test]
fn run_refuses_a_module_declaring_main_of_another_shape_by_naming_the_one_it_wants() {
    let example = Example::new("fn main() -> Int {\n    7\n}\n");

    let run = lumen_finding(&["run", example.path.to_str().expect("a UTF-8 path")], None);

    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(
        run.stderr.contains("fn main() -> ()"),
        "the shape it wants: {}",
        run.stderr
    );
}

#[test]
fn run_says_where_it_looked_when_java_home_names_no_jdk() {
    let example = Example::new(PROGRAM);
    let nowhere = example.directory.join("nowhere");

    let run = lumen_finding(
        &["run", example.path.to_str().expect("a UTF-8 path")],
        Some(&nowhere),
    );

    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(run.stderr.contains("names no JDK"), "{}", run.stderr);
    assert!(
        run.stderr.contains("nowhere"),
        "it says where it looked: {}",
        run.stderr
    );
}

/// Every file written beside the example, by the path it was written at.
fn written_by(example: &Example) -> Vec<String> {
    let mut found = Vec::new();
    beneath(&example.directory, "", &mut found);
    found.sort();
    found
}

fn beneath(directory: &Path, package: &str, found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.path().is_dir() {
            beneath(&entry.path(), &format!("{package}{name}/"), found);
        } else {
            found.push(format!("{package}{name}"));
        }
    }
}

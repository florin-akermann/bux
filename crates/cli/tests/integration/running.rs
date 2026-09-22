//! `lumen run`: a source file compiled and then run, which `docs/specs/run.md` states.

use std::path::Path;

use lumen_ir::THE_ONE_SHAPE;

use crate::common::{Example, Run, jdk, lumen, lumen_finding};

/// A program: a module declaring the `main` a run starts at, over a type and a loop.
const PROGRAM: &str = "fn main(arguments: List<String>) -> Int {\n    0\n}\n\n// example: counted([User { id: 1, active: true }]) == 1\nfn counted(users: List<User>) -> Int {\n    var total = 0\n    for user in users {\n        if user.active {\n            total += 1\n        }\n    }\n    total\n}\n\ntype User = {\n    id: Int\n    active: Bool\n}\n";

/// A library: a module the compiler accepts, and that a run has nothing to start.
const LIBRARY: &str = "// example: counted() == 2\nfn counted() -> Int {\n    2\n}\n";

/// A program the compiler will not have, which a run reports rather than starts.
const REFUSED: &str = "fn main(arguments: List<String>) -> Int {\n    missing()\n    0\n}\n";

/// A program that writes each word it was run with, one to a line, and ends with `0`.
const WRITES_ITS_ARGUMENTS: &str = "import io\n\nfn main(arguments: List<String>) -> Int {\n    for argument in arguments {\n        io.println(argument)\n    }\n    0\n}\n";

#[test]
fn run_compiles_a_program_and_runs_it_to_the_end() {
    let Some(run) = ran(PROGRAM, &[]) else {
        return;
    };

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
    let run = refused_as_no_program(LIBRARY);

    assert!(run.stderr.contains("does not declare"), "{}", run.stderr);
    assert!(
        run.stderr.contains(THE_ONE_SHAPE),
        "it names the shape to write: {}",
        run.stderr
    );
}

#[test]
fn run_refuses_a_module_declaring_main_of_another_shape_by_naming_the_one_it_wants() {
    let run = refused_as_no_program("fn main() -> Int {\n    7\n}\n");

    assert!(
        run.stderr.contains(THE_ONE_SHAPE),
        "the shape it wants: {}",
        run.stderr
    );
}

#[test]
fn run_refuses_a_module_declaring_the_main_that_took_nothing_and_gave_back_nothing() {
    let run = refused_as_no_program("fn main() -> () {\n    ()\n}\n");

    assert!(
        run.stderr.contains(THE_ONE_SHAPE),
        "the shape it wants: {}",
        run.stderr
    );
}

#[test]
fn run_refuses_a_program_the_compiler_will_not_have_and_runs_nothing() {
    let example = Example::new(REFUSED);

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
    let example = Example::new(REFUSED);

    let run = lumen_finding(&["run", example.path.to_str().expect("a UTF-8 path")], None);

    assert_eq!(run.code, 1, "{}", run.stderr);
    assert!(run.stderr.starts_with("error[L03"), "{}", run.stderr);
}

#[test]
fn run_passes_every_word_after_the_file_to_the_program_in_order_and_nothing_else() {
    let Some(run) = ran(WRITES_ITS_ARGUMENTS, &["one", "two"]) else {
        return;
    };

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(
        run.stdout, "one\ntwo\n",
        "the words themselves, and no program name in front of them"
    );
}

#[test]
fn run_passes_no_argument_at_all_where_the_command_wrote_none() {
    let Some(run) = ran(WRITES_ITS_ARGUMENTS, &[]) else {
        return;
    };

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(
        run.stdout, "",
        "an empty list is what the program is handed"
    );
}

#[test]
fn run_ends_with_the_status_the_program_gave_back() {
    let Some(run) = ran("fn main(arguments: List<String>) -> Int {\n    3\n}\n", &[]) else {
        return;
    };

    assert_eq!(run.code, 3, "{}", run.stderr);
}

#[test]
fn run_ends_with_the_low_eight_bits_of_an_answer_too_wide_for_a_status() {
    let Some(run) = ran(
        "fn main(arguments: List<String>) -> Int {\n    0 - 1\n}\n",
        &[],
    ) else {
        return;
    };

    assert_eq!(
        run.code, 255,
        "every Int maps to one status: {}",
        run.stderr
    );
}

#[test]
fn run_passes_what_the_program_writes_to_standard_error_through_untouched() {
    let source = "import io\n\nfn main(arguments: List<String>) -> Int {\n    io.eprintln(\"went wrong\")\n    0\n}\n";
    let Some(run) = ran(source, &[]) else {
        return;
    };

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.stderr, "went wrong\n");
    assert_eq!(run.stdout, "", "standard output is a channel of its own");
}

#[test]
fn run_passes_a_word_the_runner_would_otherwise_read_to_the_program_unchanged() {
    let Some(run) = ran(WRITES_ITS_ARGUMENTS, &["--help"]) else {
        return;
    };

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(
        run.stdout, "--help\n",
        "a run has no help flag of its own: the word belongs to the program"
    );
}

#[test]
fn the_topic_a_run_has_no_flag_for_is_read_through_help_instead() {
    let run = lumen(&["help", "run"]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.contains("Exit codes"), "{}", run.stdout);
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

/// What running a module of `source` with `arguments` amounted to.
///
/// Running needs a JDK and the suite does not, so this gives back nothing and says so by name
/// when `JAVA_HOME` names none, and a run that proves less says which tests it did not reach.
fn ran(source: &str, arguments: &[&str]) -> Option<Run> {
    if jdk().is_none() {
        eprintln!("skipped: JAVA_HOME names no JDK, and running a program needs one");
        return None;
    }
    let example = Example::new(source);
    let mut command = vec!["run", example.path.to_str().expect("a UTF-8 path")];
    command.extend(arguments);
    Some(lumen(&command))
}

/// What `lumen run` said about a module it has nothing to start, which it says before it looks
/// for a JDK.
fn refused_as_no_program(source: &str) -> Run {
    let example = Example::new(source);

    let run = lumen_finding(&["run", example.path.to_str().expect("a UTF-8 path")], None);

    assert_eq!(run.code, 2, "{}", run.stderr);
    run
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

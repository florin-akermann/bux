//! The behaviours of `lumen test`: what a run of a module's examples says, and how it exits.

use crate::common::{Example, Run, jdk, lumen, lumen_finding};

/// A module whose one example holds, which is the whole of what a run should find.
const HOLDS: &str = concat!(
    "// example: shared(total: 7) == 7\n",
    "fn shared(total: Int) -> Int {\n    total\n}\n",
);

/// The same module, with the one example stating what the function does not work out.
const DOES_NOT_HOLD: &str = concat!(
    "// example: shared(total: 7) == 8\n",
    "fn shared(total: Int) -> Int {\n    total\n}\n",
);

/// A module that writes a line of its own from inside the example that tries it.
///
/// The line reads exactly as a run's report of the first example, and the example holds.
const WRITES_A_LINE_ITSELF: &str = concat!(
    "import io\n\n",
    "// example: is_ready()\n",
    "fn is_ready() -> Bool {\n    io.println(\"0\")\n    true\n}\n",
);

#[test]
fn test_says_nothing_and_exits_well_when_every_example_held() {
    let Some(run) = running_the_examples(HOLDS) else {
        return;
    };

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "");
    assert_eq!(run.stderr, "");
}

#[test]
fn test_reports_the_example_that_did_not_hold_where_it_is_written() {
    let Some(run) = running_the_examples(DOES_NOT_HOLD) else {
        return;
    };

    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(
        run.stderr
            .contains("error[L0603]: the example of `shared` does not hold"),
        "{}",
        run.stderr
    );
    assert!(
        run.stderr.contains("// example: shared(total: 7) == 8"),
        "{}",
        run.stderr
    );
}

#[test]
fn a_line_a_program_writes_itself_is_not_read_as_an_example_that_did_not_hold() {
    let Some(run) = running_the_examples(WRITES_A_LINE_ITSELF) else {
        return;
    };

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.stderr, "");
}

#[test]
fn test_refuses_a_function_that_states_no_example_before_it_looks_for_a_jdk() {
    let run = running_without_a_jdk("fn shared(total: Int) -> Int {\n    total\n}\n");

    assert_eq!(run.code, 1, "{}", run.stderr);
    assert!(
        run.stderr
            .contains("error[L0601]: `shared` states no example"),
        "{}",
        run.stderr
    );
}

#[test]
fn test_refuses_a_module_that_declares_the_name_a_run_reaches_for() {
    let declaring_io = format!(
        "{HOLDS}\n// example: io(total: 7) == 7\nfn io(total: Int) -> Int {{\n    total\n}}\n"
    );

    let run = running_without_a_jdk(&declaring_io);

    assert_eq!(run.code, 1, "{}", run.stderr);
    assert!(run.stderr.contains("error[L0604]"), "{}", run.stderr);
}

#[test]
fn test_says_nothing_and_exits_well_for_a_module_that_states_no_example() {
    let run = running_without_a_jdk("type Colour =\n    | Red\n    | Blue\n");

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.stderr, "");
}

#[test]
fn test_says_which_variable_names_the_jdk_when_nothing_does() {
    let run = running_without_a_jdk(HOLDS);

    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(
        run.stderr.contains("JAVA_HOME is not set"),
        "{}",
        run.stderr
    );
}

/// One run of `lumen test` over `source`, or nothing where `JAVA_HOME` names no JDK.
fn running_the_examples(source: &str) -> Option<Run> {
    if jdk().is_none() {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the examples needs one");
        return None;
    }
    let example = Example::new(source);
    Some(lumen(&["test", named(&example)]))
}

/// One run of `lumen test` over `source`, as if `JAVA_HOME` named nothing at all.
///
/// Everything a run refuses before it starts a JVM is refused the same either way, and asking
/// for no JDK is how a test says that the JDK is not what it is about.
fn running_without_a_jdk(source: &str) -> Run {
    let example = Example::new(source);
    lumen_finding(&["test", named(&example)], None)
}

fn named(example: &Example) -> &str {
    example.path.to_str().expect("a path of text")
}

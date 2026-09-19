//! The executable examples under `tests/spec/`, each held to what it says about itself.
//!
//! `docs/specs/executable-examples.md` says what an example is and what its header means. This is
//! the one place that walks them, so an example is never in the tree without being run.

use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use lumen_diagnostics::Code;
use lumen_format::format;

use crate::common::{Example, jdk, lumen};

#[test]
fn every_example_is_accepted_refused_or_run_exactly_as_it_says() {
    let examples = examples();
    assert!(
        !examples.is_empty(),
        "tests/spec holds at least one example"
    );
    for path in examples {
        hold_to_its_expectation(&path);
    }
}

/// Canonical form is not what stops a refused example, so its header names the real refusal.
#[test]
fn every_example_that_parses_is_in_canonical_form() {
    for path in examples() {
        let source = read_to_string(&path).expect("an example is readable");
        let Ok(canonical) = format(&source) else {
            continue;
        };
        assert_eq!(
            canonical,
            source,
            "{} is not in canonical form",
            path.display()
        );
    }
}

/// Every example under `tests/spec/`, in a stable order.
fn examples() -> Vec<PathBuf> {
    let mut found = Vec::new();
    let spec = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/spec")
        .canonicalize()
        .expect("tests/spec exists");
    collect(&spec, &mut found);
    found.sort();
    found
}

fn collect(directory: &Path, found: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", directory.display()));
    for entry in entries {
        let path = entry.expect("a directory entry is readable").path();
        if path.is_dir() {
            collect(&path, found);
        } else if path.extension().is_some_and(|kind| kind == "lm") {
            found.push(path);
        }
    }
}

fn hold_to_its_expectation(path: &Path) {
    let source = read_to_string(path).expect("an example is readable");
    let run = lumen(&["check", path.to_str().expect("a UTF-8 path")]);
    let shown = path.display();
    match Expectation::of(&source, path) {
        Expectation::Compiles => {
            assert_eq!(run.code, 0, "{shown} does not compile:\n{}", run.stderr);
        }
        Expectation::Refused(code) => {
            let opening = format!("error[{}]: ", code.number());
            assert_eq!(run.code, 1, "{shown} is not refused:\n{}", run.stderr);
            assert!(
                run.stderr.starts_with(&opening),
                "{shown} expects {opening}but said:\n{}",
                run.stderr
            );
        }
        Expectation::Runs => started(&source, path),
    }
}

/// An example that is run is copied out of the tree first, so a run writes nothing into it.
fn started(source: &str, path: &Path) {
    if jdk().is_none() {
        eprintln!(
            "skipped: {}: JAVA_HOME names no JDK, and an example that is run needs one",
            path.display()
        );
        return;
    }
    let copy = Example::new(source);
    let run = lumen(&["run", copy.path.to_str().expect("a UTF-8 path")]);
    assert_eq!(
        run.code,
        0,
        "{} does not run to the end:\n{}",
        path.display(),
        run.stderr
    );
}

/// What an example says about itself.
#[derive(Debug, PartialEq, Eq)]
enum Expectation {
    /// The file carries no header, so the compiler must accept it.
    Compiles,
    /// The file opens with `// expect-error:` and the code it must be refused with.
    Refused(Code),
    /// The file opens with `// expect-run`, so it must compile and then run to the end.
    Runs,
}

impl Expectation {
    fn of(source: &str, path: &Path) -> Self {
        let first = source.lines().next().unwrap_or_default().trim_end();
        if first == RUNS {
            return Self::Runs;
        }
        let Some(written) = source
            .lines()
            .next()
            .and_then(|first| first.strip_prefix(HEADER))
        else {
            return Self::Compiles;
        };
        let written = written.trim();
        Code::written_as(written).map_or_else(
            || panic!("{}: there is no diagnostic {written}", path.display()),
            Self::Refused,
        )
    }
}

/// What an example writes to name the diagnostic it is refused with.
const HEADER: &str = "// expect-error:";

/// What an example writes to say it is run, and must run to the end.
const RUNS: &str = "// expect-run";

#[test]
fn an_example_with_no_header_compiles() {
    let source = "// an ordinary comment\nimport io\n";

    assert_eq!(
        Expectation::of(source, Path::new("demo.lm")),
        Expectation::Compiles
    );
}

#[test]
fn an_example_that_is_refused_names_the_diagnostic_on_its_first_line() {
    let source = "// expect-error: L0105\nfn f() {\n}\n";

    assert_eq!(
        Expectation::of(source, Path::new("demo.lm")),
        Expectation::Refused(Code::ChainedComparison)
    );
}

#[test]
#[should_panic(expected = "demo.lm: there is no diagnostic L9999")]
fn an_example_may_not_name_a_diagnostic_the_compiler_cannot_raise() {
    let _ = Expectation::of("// expect-error: L9999\n", Path::new("demo.lm"));
}

#[test]
fn an_example_that_is_run_says_so_on_its_first_line() {
    let source = "// expect-run\nfn main() -> () {\n    ()\n}\n";

    assert_eq!(
        Expectation::of(source, Path::new("demo.lm")),
        Expectation::Runs
    );
}

//! The executable examples under `tests/spec/`, each held to what it says about itself.
//!
//! `docs/specs/executable-examples.md` says what an example is and what its header means. This is
//! the one place that walks them, so an example is never in the tree without being run.

use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use lumen_diagnostics::Code;
use lumen_format::format;

use crate::common::{Example, Sibling, jdk, lumen};

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

/// An example as the harness holds it: the text it is, and the path that names it in a failure.
///
/// The two travel together, because everything an example says about itself is read out of its
/// text and reported against its path.
struct Opened<'a> {
    source: &'a str,
    path: &'a Path,
}

fn hold_to_its_expectation(path: &Path) {
    let source = read_to_string(path).expect("an example is readable");
    let opened = Opened {
        source: &source,
        path,
    };
    let run = lumen(&["check", path.to_str().expect("a UTF-8 path")]);
    let shown = path.display();
    match Expectation::of(&opened) {
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
        Expectation::Runs(written) => started(&opened, &written),
    }
}

/// An example that is run is copied out of the tree first, so a run writes nothing into it.
///
/// Every module beside it is copied too, under its own name, because an import names the file
/// beside the one that writes it and an example may be written across several.
fn started(opened: &Opened, written: &str) {
    let shown = opened.path.display();
    if jdk().is_none() {
        eprintln!("skipped: {shown}: JAVA_HOME names no JDK, and an example that is run needs one");
        return;
    }
    let copy = Example::new(opened.source);
    for (named, source) in beside(opened.path) {
        copy.beside_it(&Sibling {
            named: &named,
            content: &source,
        });
    }
    let run = lumen(&["run", copy.path.to_str().expect("a UTF-8 path")]);
    assert_eq!(
        run.code, 0,
        "{shown} does not run to the end:\n{}",
        run.stderr
    );
    assert_eq!(
        run.stdout, written,
        "{shown} writes {:?} where its header states {written:?}",
        run.stdout
    );
}

/// Every other module in the example's own directory, each under the name its file gives it.
fn beside(path: &Path) -> Vec<(String, String)> {
    let directory = path.parent().expect("an example sits in a directory");
    let mut found = Vec::new();
    for entry in std::fs::read_dir(directory).expect("an example's directory is readable") {
        let beside = entry.expect("a directory entry is readable").path();
        if beside == path || beside.extension().is_none_or(|kind| kind != "lm") {
            continue;
        }
        let named = beside
            .file_stem()
            .expect("a module is named by its file")
            .to_string_lossy()
            .into_owned();
        found.push((
            named,
            read_to_string(&beside).expect("a module is readable"),
        ));
    }
    found.sort();
    found
}

/// What an example says about itself.
#[derive(Debug, PartialEq, Eq)]
enum Expectation {
    /// The file carries no header, so the compiler must accept it.
    Compiles,
    /// The file opens with `// expect-error:` and the code it must be refused with.
    Refused(Code),
    /// The file opens with `// expect-run`, so it must compile, run to the end, and write
    /// exactly the lines stated under the header.
    Runs(String),
}

impl Expectation {
    fn of(opened: &Opened) -> Self {
        let source = opened.source;
        let first = source.lines().next().unwrap_or_default().trim_end();
        if first == RUNS {
            return Self::Runs(written_out(opened));
        }
        states_nothing_written(opened);
        let Some(written) = source
            .lines()
            .next()
            .and_then(|first| first.strip_prefix(HEADER))
        else {
            return Self::Compiles;
        };
        let written = written.trim();
        Code::written_as(written).map_or_else(
            || {
                panic!(
                    "{}: there is no diagnostic {written}",
                    opened.path.display()
                )
            },
            Self::Refused,
        )
    }
}

/// What an example writes to name the diagnostic it is refused with.
const HEADER: &str = "// expect-error:";

/// What an example writes to say it is run, and must run to the end.
const RUNS: &str = "// expect-run";

/// What an example writes before each line the program must write.
const WRITES: &str = "// >";

/// The text the lines under the header say the program writes, each with its line break.
///
/// They come directly under the header, so the first line that is not one of them ends the
/// output an example states, and an example that states none of them writes nothing at all.
/// A line below that block which still opens with the marker is a line the author meant to
/// state and the harness would not have compared, so it fails the run rather than being read
/// as a program that wrote the wrong thing.
fn written_out(opened: &Opened) -> String {
    let lines: Vec<&str> = opened.source.lines().skip(1).collect();
    let stated: Vec<&str> = lines.iter().copied().map_while(stated).collect();
    assert!(
        !lines[stated.len()..].iter().copied().any(marked),
        "{}: a line stating output comes directly under the header, and this one does not",
        opened.path.display()
    );
    stated.iter().fold(String::new(), |mut written, line| {
        written.push_str(line);
        written.push('\n');
        written
    })
}

/// An example that is not run states no output, because nothing would ever compare it.
fn states_nothing_written(opened: &Opened) {
    assert!(
        !opened.source.lines().any(marked),
        "{}: only an example that is run states what it writes",
        opened.path.display()
    );
}

/// Whether `line` opens the way a line stating output does, however it goes on.
fn marked(line: &str) -> bool {
    line.starts_with(WRITES)
}

/// The line `written` states the program writes, where it states one.
///
/// A line is written after a space, which canonical form does not keep where there is nothing
/// after it, so a line the program writes empty is stated with the marker and nothing else.
fn stated(written: &str) -> Option<&str> {
    let rest = written.strip_prefix(WRITES)?;
    if rest.is_empty() {
        return Some(rest);
    }
    rest.strip_prefix(' ')
}

/// What `source` says about itself, under a name a failure can print.
fn said_by(source: &str) -> Expectation {
    Expectation::of(&Opened {
        source,
        path: Path::new("demo.lm"),
    })
}

#[test]
fn an_example_with_no_header_compiles() {
    assert_eq!(
        said_by("// an ordinary comment\nimport io\n"),
        Expectation::Compiles
    );
}

#[test]
fn an_example_that_is_refused_names_the_diagnostic_on_its_first_line() {
    assert_eq!(
        said_by("// expect-error: L0105\nfn f() {\n}\n"),
        Expectation::Refused(Code::ChainedComparison)
    );
}

#[test]
#[should_panic(expected = "demo.lm: there is no diagnostic L9999")]
fn an_example_may_not_name_a_diagnostic_the_compiler_cannot_raise() {
    let _ = said_by("// expect-error: L9999\n");
}

#[test]
fn an_example_that_is_run_says_so_on_its_first_line() {
    assert_eq!(
        said_by("// expect-run\nfn main() -> () {\n    ()\n}\n"),
        Expectation::Runs(String::new())
    );
}

#[test]
fn an_example_that_writes_nothing_states_nothing_which_is_as_much_a_claim_as_any_other() {
    assert_eq!(
        said_by("// expect-run\n// an ordinary comment\nfn main() -> () {\n}\n"),
        Expectation::Runs(String::new())
    );
}

#[test]
fn an_example_states_each_line_it_writes_under_the_header_and_each_ends_with_a_line_break() {
    assert_eq!(
        said_by("// expect-run\n// > one\n// > two\nfn main() -> () {\n}\n"),
        Expectation::Runs("one\ntwo\n".to_owned())
    );
}

#[test]
fn a_line_an_example_states_is_taken_as_it_is_written_spaces_and_all() {
    assert_eq!(
        said_by("// expect-run\n// >   held  \nfn main() -> () {\n}\n"),
        Expectation::Runs("  held  \n".to_owned())
    );
}

#[test]
fn a_line_the_program_writes_empty_is_stated_with_the_marker_and_nothing_after_it() {
    assert_eq!(
        said_by("// expect-run\n// > one\n// >\n// > two\nfn main() -> () {\n}\n"),
        Expectation::Runs("one\n\ntwo\n".to_owned())
    );
}

#[test]
fn a_line_an_example_states_may_itself_begin_with_the_marker_it_is_stated_after() {
    assert_eq!(
        said_by("// expect-run\n// > > held\nfn main() -> () {\n}\n"),
        Expectation::Runs("> held\n".to_owned())
    );
}

#[test]
fn an_ordinary_comment_below_the_stated_lines_ends_what_an_example_says_it_writes() {
    assert_eq!(
        said_by("// expect-run\n// > one\n// a comment\nfn main() -> () {\n}\n"),
        Expectation::Runs("one\n".to_owned())
    );
}

#[test]
#[should_panic(expected = "demo.lm: a line stating output comes directly under the header")]
fn a_marker_with_no_space_after_it_fails_rather_than_stating_one_line_fewer() {
    let _ = said_by("// expect-run\n// >one\nfn main() -> () {\n}\n");
}

#[test]
#[should_panic(expected = "demo.lm: a line stating output comes directly under the header")]
fn a_line_stating_output_below_the_block_fails_rather_than_being_left_uncompared() {
    let _ = said_by("// expect-run\n// > one\n// a note\n// > two\nfn main() -> () {\n}\n");
}

#[test]
#[should_panic(expected = "demo.lm: only an example that is run states what it writes")]
fn an_example_that_is_refused_may_not_state_output_that_nothing_would_compare() {
    let _ = said_by("// expect-error: L0105\n// > one\nfn f() {\n}\n");
}

#[test]
#[should_panic(expected = "demo.lm: only an example that is run states what it writes")]
fn an_example_with_no_header_may_not_state_output_either() {
    let _ = said_by("// > one\nimport io\n");
}

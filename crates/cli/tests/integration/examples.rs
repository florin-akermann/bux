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
        Expectation::Runs(stated) => started(&opened, &stated),
    }
}

/// An example that is run is copied out of the tree first, so a run writes nothing into it.
///
/// Every module beside it is copied too, under its own name, because an import names the file
/// beside the one that writes it and an example may be written across several.
fn started(opened: &Opened, stated: &Ran) {
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
    let mut command = vec!["run", copy.path.to_str().expect("a UTF-8 path")];
    command.extend(stated.arguments.iter().map(String::as_str));
    let run = lumen(&command);
    assert_eq!(
        run.code, stated.status,
        "{shown} ends with {} where its header states {}:\n{}",
        run.code, stated.status, run.stderr
    );
    assert_eq!(
        run.stdout, stated.output,
        "{shown} writes {:?} where its header states {:?}",
        run.stdout, stated.output
    );
    assert_eq!(
        run.stderr, stated.errors,
        "{shown} writes {:?} to standard error where its header states {:?}",
        run.stderr, stated.errors
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
    /// The file opens with `// expect-run`, so it must compile and run exactly as the header
    /// and the lines under it state.
    Runs(Ran),
}

/// What an example headed `// expect-run` says its run amounts to.
#[derive(Debug, Default, PartialEq, Eq)]
struct Ran {
    /// The words written after the header, which reach the program as the list `main` takes.
    arguments: Vec<String>,
    /// What it writes to standard output, each line with its line break.
    output: String,
    /// What it writes to standard error, each line with its line break.
    errors: String,
    /// The status it ends with, which is `0` where the header states none.
    status: i32,
}

impl Expectation {
    fn of(opened: &Opened) -> Self {
        let source = opened.source;
        let first = source.lines().next().unwrap_or_default().trim_end();
        if let Some(arguments) = run_with(first) {
            return Self::Runs(ran(opened, arguments));
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

/// What an example writes to say it is run, with the words it is run with after it.
const RUNS: &str = "// expect-run";

/// What an example writes before each line the program writes to standard output.
const WRITES: &str = "// >";

/// What an example writes before each line the program writes to standard error.
const COMPLAINS: &str = "// !";

/// What an example writes before the status the program ends with.
const ENDS: &str = "// status";

/// The words `first` says the program is run with, where `first` is the header at all.
///
/// Every word after the header is one argument, in the order the program is handed them, and a
/// header with nothing after it runs the program with none.
fn run_with(first: &str) -> Option<Vec<String>> {
    let rest = first.strip_prefix(RUNS)?;
    if !rest.is_empty() && !rest.starts_with(' ') {
        return None;
    }
    Some(rest.split_whitespace().map(str::to_owned).collect())
}

/// What the lines under the header say the run amounts to.
///
/// They come directly under the header, so the first line that is not one of them ends what an
/// example states, and an example that states none of them writes nothing and ends with `0`.
/// A line below that block which still opens with a marker is a line the author meant to state
/// and the harness would not have compared, so it fails the run rather than being read as a
/// program that did the wrong thing.
fn ran(opened: &Opened, arguments: Vec<String>) -> Ran {
    let lines: Vec<&str> = opened.source.lines().skip(1).collect();
    let stated: Vec<Stated<'_>> = lines.iter().copied().map_while(states).collect();
    assert!(
        !lines[stated.len()..].iter().copied().any(marked),
        "{}: a line stating a run comes directly under the header, and this one does not",
        opened.path.display()
    );
    stated.iter().fold(
        Ran {
            arguments,
            ..Ran::default()
        },
        Stated::added_to,
    )
}

/// An example that is not run states nothing about a run, because nothing would compare it.
fn states_nothing_written(opened: &Opened) {
    assert!(
        !opened.source.lines().any(marked),
        "{}: only an example that is run states what it writes",
        opened.path.display()
    );
}

/// Whether `line` opens the way a line stating a run does, however it goes on.
///
/// A marker ends at the end of the line or at a space, so an ordinary comment whose first word
/// merely opens with one, such as `// statuses are read elsewhere`, states nothing about a run.
fn marked(line: &str) -> bool {
    [WRITES, COMPLAINS, ENDS]
        .iter()
        .filter_map(|marker| line.strip_prefix(marker))
        .any(|rest| after_the_marker(rest).is_some())
}

/// One line under the header, as what it says about the run.
#[derive(Debug, PartialEq, Eq)]
enum Stated<'a> {
    /// One line the program writes to standard output.
    Output(&'a str),
    /// One line the program writes to standard error.
    Errors(&'a str),
    /// The status the program ends with.
    Status(i32),
}

impl Stated<'_> {
    /// `ran` with what this line states added to it.
    fn added_to(mut ran: Ran, stated: &Self) -> Ran {
        match stated {
            Self::Output(line) => written(&mut ran.output, line),
            Self::Errors(line) => written(&mut ran.errors, line),
            Self::Status(status) => ran.status = *status,
        }
        ran
    }
}

/// Adds `line`, and the line break after it, to what the program writes on one channel.
fn written(channel: &mut String, line: &str) {
    channel.push_str(line);
    channel.push('\n');
}

/// What `line` states about the run, where it states anything at all.
fn states(line: &str) -> Option<Stated<'_>> {
    if let Some(rest) = line.strip_prefix(WRITES) {
        return after_the_marker(rest).map(Stated::Output);
    }
    if let Some(rest) = line.strip_prefix(COMPLAINS) {
        return after_the_marker(rest).map(Stated::Errors);
    }
    let rest = line.strip_prefix(ENDS)?;
    rest.strip_prefix(' ')?.parse().ok().map(Stated::Status)
}

/// The line written after a marker, where the marker is followed the way it has to be.
///
/// A line is written after a space, which canonical form does not keep where there is nothing
/// after it, so a line the program writes empty is stated with the marker and nothing else.
/// Anything else after the marker leaves it another word, and the line an ordinary comment.
fn after_the_marker(rest: &str) -> Option<&str> {
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

/// An example headed `// expect-run`, stating `stated` about a run that does nothing at all.
fn run_of(stated: &str) -> String {
    format!("// expect-run\n{stated}fn main(arguments: List<String>) -> Int {{\n}}\n")
}

#[test]
fn an_example_that_is_run_says_so_on_its_first_line() {
    assert_eq!(said_by(&run_of("")), Expectation::Runs(Ran::default()));
}

#[test]
fn an_example_that_writes_nothing_states_nothing_which_is_as_much_a_claim_as_any_other() {
    assert_eq!(
        said_by(&run_of("// an ordinary comment\n")),
        Expectation::Runs(Ran::default())
    );
}

#[test]
fn an_example_states_each_line_it_writes_under_the_header_and_each_ends_with_a_line_break() {
    assert_eq!(
        said_by(&run_of("// > one\n// > two\n")),
        Expectation::Runs(Ran {
            output: "one\ntwo\n".to_owned(),
            ..Ran::default()
        })
    );
}

#[test]
fn an_example_states_each_line_it_writes_to_standard_error_after_its_own_marker() {
    assert_eq!(
        said_by(&run_of("// > said\n// ! went wrong\n")),
        Expectation::Runs(Ran {
            output: "said\n".to_owned(),
            errors: "went wrong\n".to_owned(),
            ..Ran::default()
        })
    );
}

#[test]
fn an_example_states_the_status_the_program_ends_with_where_it_is_not_zero() {
    assert_eq!(
        said_by(&run_of("// status 3\n")),
        Expectation::Runs(Ran {
            status: 3,
            ..Ran::default()
        })
    );
}

#[test]
fn an_example_is_run_with_every_word_after_the_header_in_the_order_it_writes_them() {
    assert_eq!(
        said_by("// expect-run one two\nfn main(arguments: List<String>) -> Int {\n}\n"),
        Expectation::Runs(Ran {
            arguments: vec!["one".to_owned(), "two".to_owned()],
            ..Ran::default()
        })
    );
}

#[test]
fn a_line_an_example_states_is_taken_as_it_is_written_spaces_and_all() {
    assert_eq!(
        said_by(&run_of("// >   held  \n")),
        Expectation::Runs(Ran {
            output: "  held  \n".to_owned(),
            ..Ran::default()
        })
    );
}

#[test]
fn a_line_the_program_writes_empty_is_stated_with_the_marker_and_nothing_after_it() {
    assert_eq!(
        said_by(&run_of("// > one\n// >\n// > two\n")),
        Expectation::Runs(Ran {
            output: "one\n\ntwo\n".to_owned(),
            ..Ran::default()
        })
    );
}

#[test]
fn a_line_an_example_states_may_itself_begin_with_the_marker_it_is_stated_after() {
    assert_eq!(
        said_by(&run_of("// > > held\n")),
        Expectation::Runs(Ran {
            output: "> held\n".to_owned(),
            ..Ran::default()
        })
    );
}

#[test]
fn an_ordinary_comment_below_the_stated_lines_ends_what_an_example_says_it_writes() {
    assert_eq!(
        said_by(&run_of("// > one\n// a comment\n")),
        Expectation::Runs(Ran {
            output: "one\n".to_owned(),
            ..Ran::default()
        })
    );
}

#[test]
fn a_marker_with_no_space_after_it_is_an_ordinary_comment_rather_than_a_marker() {
    assert_eq!(
        said_by(&run_of("// >one\n")),
        Expectation::Runs(Ran::default())
    );
}

#[test]
fn a_comment_whose_first_word_merely_opens_with_a_marker_states_nothing_about_a_run() {
    assert_eq!(
        said_by(&run_of("// statuses are read elsewhere\n")),
        Expectation::Runs(Ran::default())
    );
}

#[test]
#[should_panic(expected = "demo.lm: a line stating a run comes directly under the header")]
fn a_line_stating_output_below_the_block_fails_rather_than_being_left_uncompared() {
    let _ = said_by(&run_of("// > one\n// a note\n// > two\n"));
}

#[test]
#[should_panic(expected = "demo.lm: a line stating a run comes directly under the header")]
fn a_status_that_is_no_whole_number_fails_rather_than_being_read_as_a_comment() {
    let _ = said_by(&run_of("// status soon\n"));
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

#[test]
#[should_panic(expected = "demo.lm: only an example that is run states what it writes")]
fn an_example_with_no_header_may_not_state_a_status_either() {
    let _ = said_by("// status 1\nimport io\n");
}

#[test]
fn an_example_with_no_header_may_write_a_comment_that_merely_opens_with_a_marker() {
    assert_eq!(
        said_by("// statuses are read elsewhere\nimport io\n"),
        Expectation::Compiles
    );
}

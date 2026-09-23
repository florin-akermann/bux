//! The lexer written in Bux, `compiler/lexer.lm`, held to the Rust lexer's tokens.
//!
//! `docs/specs/lexer.md` states the behaviour, and the Rust lexer is the answer the Bux one must
//! give. Building the Bux lexer needs no JDK; running what was built needs one, and a test that
//! runs it is skipped with a named reason when `JAVA_HOME` names none.

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use hegel::TestCase;
use hegel::generators as gs;
use lumen_lexer::{TokenKind, lex};

use crate::common::{Example, Sibling, answers_of_one_run, as_argument, jdk, lumen, repository};

/// A program that writes, beside each file it is run with, its tokens as the Bux lexer lists them.
///
/// One run lexes every file, because each run is one compiler and one JVM.
const WRITES_THE_TOKENS: &str = "import files\n\nimport lexer\n\nfn main(arguments: List<String>) -> Int {\n    for path in arguments {\n        _ = files.write(path + \".answer\", lexer.listed(ok_or(files.read(path), \"\")))\n    }\n    0\n}\n";

/// How many runs the property makes, which is one compiler and one JVM each.
const CASES: u64 = 10;

/// How many drawn sources one run lexes.
const SOURCES_A_RUN: usize = 8;

/// What a drawn source is made of: a piece of every token class, and each character that ends
/// one, starts one, or is two bytes or four in UTF-8.
const PIECES: [&str; 22] = [
    "fn", "fnord", "_", "x1", "123", "\"", "\\", "//", ":=", ":", "=", "-", ">", "&", "@", " ",
    "\t", "\r", "\n", "é", "€", "😀",
];

#[test]
fn the_bux_lexer_builds_under_the_rust_compiler() {
    let program = the_bux_lexer_beside(WRITES_THE_TOKENS);

    let run = lumen(&["build", as_argument(&program.path)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn every_example_the_bux_lexer_states_holds() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the examples needs one");
        return;
    };

    let run = lumen(&["test", as_argument(&bux_lexer())]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn the_bux_lexer_gives_the_rust_lexers_tokens_on_every_lexer_example() {
    let examples = lexer_examples();
    assert!(!examples.is_empty(), "tests/spec/lexer holds an example");

    let sources: Vec<String> = examples
        .iter()
        .map(|example| fs::read_to_string(example).expect("an example is UTF-8"))
        .collect();

    let Some(written) = listed_by_the_bux_lexer(&sources) else {
        return;
    };

    for ((example, source), written) in examples.iter().zip(&sources).zip(written) {
        assert_eq!(written, listed(source), "{}", example.display());
    }
}

/// `docs/specs/lexer.md`: the Bux lexer and the Rust lexer give one answer on any text.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn the_bux_lexer_gives_the_rust_lexers_tokens_on_drawn_text(tc: TestCase) {
    let sources: Vec<String> = (0..SOURCES_A_RUN).map(|_| drawn_source(&tc)).collect();

    let Some(written) = listed_by_the_bux_lexer(&sources) else {
        return;
    };

    for (source, written) in sources.iter().zip(written) {
        assert_eq!(written, listed(source), "{source:?}");
    }
}

/// A source of pieces run together, long enough that one run of the JVM meets many of them.
///
/// A case costs a build and a JVM, so each draws long sources rather than many short ones.
fn drawn_source(tc: &TestCase) -> String {
    let pieces: Vec<&str> = tc.draw(
        gs::vecs(gs::sampled_from(&PIECES))
            .min_size(16)
            .max_size(64),
    );
    pieces.concat()
}

/// What the Bux lexer lists for each source, in order, and nothing where no JDK can run it.
fn listed_by_the_bux_lexer(sources: &[String]) -> Option<Vec<String>> {
    if jdk().is_none() {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the Bux lexer needs one");
        return None;
    }
    let program = the_bux_lexer_beside(WRITES_THE_TOKENS);
    Some(answers_of_one_run(&program, sources))
}

/// A program of `content`, with the Bux lexer beside it as the module `lexer`.
fn the_bux_lexer_beside(content: &str) -> Example {
    let program = Example::new(content);
    let lexer = fs::read_to_string(bux_lexer()).expect("compiler/lexer.lm is readable");
    program.beside_it(&Sibling {
        named: "lexer",
        content: &lexer,
    });
    program
}

/// The tokens the Rust lexer gives, one line each, in the form the Bux lexer's `listed` writes.
fn listed(source: &str) -> String {
    let mut written = String::new();
    for token in lex(source) {
        let (start, end) = (token.span.start(), token.span.end());
        writeln!(written, "{} {start}..{end}", kind(token.kind)).expect("a String takes a line");
    }
    written
}

/// A kind as the Bux lexer shows it: a keyword and a punctuation by the text that spells it.
fn kind(kind: TokenKind) -> String {
    match kind {
        TokenKind::Keyword(keyword) => format!("Keyword({})", keyword.text()),
        TokenKind::Punct(punct) => format!("Punct({})", punct.text()),
        other => format!("{other:?}"),
    }
}

/// Every `.lm` example under `tests/spec/lexer`, in the order their names sort.
fn lexer_examples() -> Vec<PathBuf> {
    let mut examples: Vec<PathBuf> = fs::read_dir(repository().join("tests/spec/lexer"))
        .expect("tests/spec/lexer exists")
        .map(|entry| entry.expect("a directory entry is readable").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "lm"))
        .collect();
    examples.sort();
    examples
}

fn bux_lexer() -> PathBuf {
    repository().join("compiler/lexer.lm")
}

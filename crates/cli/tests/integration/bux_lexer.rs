//! The lexer written in Bux, `compiler/lexer.lm`, held to the Rust lexer's tokens.
//!
//! `docs/specs/lexer.md` states the behaviour, and the Rust lexer is the answer the Bux one must
//! give. Building the Bux lexer needs no JDK; running what was built needs one, and a test that
//! runs it is skipped with a named reason when `JAVA_HOME` names none.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use hegel::TestCase;
use hegel::generators as gs;
use lumen_lexer::{TokenKind, lex};

use crate::common::{Example, Sibling, Within, as_argument, jdk, lumen, repository};

/// A program that writes the tokens of each file it is run with, as the Bux lexer lists them.
const LISTS_THE_TOKENS: &str = "import files\n\nimport io\n\nimport lexer\n\nfn main(arguments: List<String>) -> Int {\n    for path in arguments {\n        match files.read(path) {\n            Ok(source) => io.print(lexer.listed(source))\n            Err(why) => io.eprintln(why)\n        }\n    }\n    0\n}\n";

/// How many drawn sources the property lexes, which is one compiler and one JVM each.
const CASES: u64 = 40;

/// What a drawn source is made of: a piece of every token class, and each character that ends
/// one, starts one, or is two bytes or four in UTF-8.
const PIECES: [&str; 22] = [
    "fn", "fnord", "_", "x1", "123", "\"", "\\", "//", ":=", ":", "=", "-", ">", "&", "@", " ",
    "\t", "\r", "\n", "é", "€", "😀",
];

#[test]
fn the_bux_lexer_builds_under_the_rust_compiler() {
    let program = the_bux_lexer_beside(LISTS_THE_TOKENS);

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

    for example in examples {
        let source = fs::read_to_string(&example).expect("an example is UTF-8");

        let Some(written) = listed_by_the_bux_lexer(&example) else {
            return;
        };

        assert_eq!(written, listed(&source), "{}", example.display());
    }
}

/// `docs/specs/lexer.md`: the Bux lexer and the Rust lexer give one answer on any text.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn the_bux_lexer_gives_the_rust_lexers_tokens_on_drawn_text(tc: TestCase) {
    let source = drawn_source(&tc);
    let drawn = Example::new("");
    drawn.within_it(&Within {
        at: "drawn.txt",
        content: &source,
    });

    let Some(written) = listed_by_the_bux_lexer(&drawn.directory.join("drawn.txt")) else {
        return;
    };

    assert_eq!(written, listed(&source), "{source:?}");
}

/// A source of pieces run together, long enough that one run of the JVM meets many of them.
///
/// A case costs a build and a JVM, so each draws a long source rather than many short ones.
fn drawn_source(tc: &TestCase) -> String {
    let pieces: Vec<&str> = tc.draw(
        gs::vecs(gs::sampled_from(&PIECES))
            .min_size(16)
            .max_size(64),
    );
    pieces.concat()
}

/// What the Bux lexer lists for the file at `path`, and nothing where no JDK is there to run it.
fn listed_by_the_bux_lexer(path: &Path) -> Option<String> {
    if jdk().is_none() {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the Bux lexer needs one");
        return None;
    }
    let program = the_bux_lexer_beside(LISTS_THE_TOKENS);

    let run = lumen(&["run", as_argument(&program.path), as_argument(path)]);

    assert_eq!(run.code, 0, "{}: {}", path.display(), run.stderr);
    Some(run.stdout)
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

//! The parser written in Bux, `compiler/parser.lm`, held to the Rust parser's printed answer.
//!
//! `docs/specs/grammar.md` states the behaviour and the printed form, and the Rust parser is the
//! answer the Bux one must give. Building the Bux parser needs no JDK; running what was built
//! needs one, and a test that runs it is skipped with a named reason when `JAVA_HOME` names none.

use std::fs;
use std::path::PathBuf;

use hegel::TestCase;
use hegel::generators as gs;

use crate::common::{Example, Sibling, answers_of_one_run, as_argument, jdk, lumen, repository};

#[path = "../../../parser/tests/integration/printed.rs"]
mod printed;

/// A program that writes, beside each file it is run with, the answer of the Bux parser.
///
/// One run parses every file, because each run is one compiler and one JVM.
const WRITES_THE_ANSWER: &str = "import files\n\nimport parser\n\nfn main(arguments: List<String>) -> Int {\n    for path in arguments {\n        _ = files.write(path + \".answer\", parser.printed(ok_or(files.read(path), \"\")))\n    }\n    0\n}\n";

/// The modules of the Bux compiler that the parser is, beside the program as siblings.
const MODULES: [&str; 3] = ["lexer", "ast", "parser"];

/// How many runs the property makes, which is one compiler and one JVM each.
const CASES: u64 = 10;

/// How many drawn sources one run parses.
const SOURCES_A_RUN: usize = 8;

/// How a drawn source opens, so that most sources reach past the first item before they fail.
const OPENINGS: [&str; 8] = [
    "",
    "fn f(a: Int) -> Int {\n",
    "type T = ",
    "extern ",
    "trait T<A> {\n",
    "fn g() {\n    match x {\n",
    "fn k() -> Int {\n    9223372036854775807 + ",
    "fn h() {\n    x := \"é\u{301}\u{85}\u{200b}\u{1}\"\n",
];

/// What a drawn source is made of: a word, a literal, or a token of each part of the grammar.
const PIECES: [&str; 47] = [
    "fn", "f", "x", "Int", "demo", "(", ")", "{", "}", "[", "]", "<", ">", ",", ":", "=", ":=",
    "+=", "->", "=>", "|", ".", "?", "!", "-", "+", "*", "==", "&&", "match", "if", "else", "for",
    "in", "var", "return", "_", "static", "int", "1", "\"a\"", "\"\\q\"", "@", " ", "\n", "\n",
    "//\n",
];

#[test]
fn the_bux_parser_builds_under_the_rust_compiler() {
    let program = the_bux_parser_beside(WRITES_THE_ANSWER);

    let run = lumen(&["build", as_argument(&program.path)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn every_example_the_bux_parser_and_its_tree_state_holds() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the examples needs one");
        return;
    };

    for module in ["ast", "parser"] {
        let run = lumen(&["test", as_argument(&compiler_module(module))]);

        assert_eq!(run.code, 0, "{module}: {}", run.stderr);
    }
}

#[test]
fn the_bux_parser_gives_the_rust_parsers_answer_on_every_parser_example() {
    let examples = parser_examples();
    assert!(!examples.is_empty(), "tests/spec/parser holds an example");

    let sources: Vec<String> = examples
        .iter()
        .map(|example| fs::read_to_string(example).expect("an example is UTF-8"))
        .collect();

    let Some(written) = printed_by_the_bux_parser(&sources) else {
        return;
    };

    for ((example, source), written) in examples.iter().zip(&sources).zip(written) {
        assert_eq!(written, answer(source), "{}", example.display());
    }
}

/// `docs/specs/grammar.md`: the Bux parser and the Rust parser give one answer on any text.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn the_bux_parser_gives_the_rust_parsers_answer_on_drawn_text(tc: TestCase) {
    let sources: Vec<String> = (0..SOURCES_A_RUN).map(|_| drawn_source(&tc)).collect();

    let Some(written) = printed_by_the_bux_parser(&sources) else {
        return;
    };

    for (source, written) in sources.iter().zip(written) {
        assert_eq!(written, answer(source), "{source:?}");
    }
}

/// An opening and pieces run together, long enough that one run of the JVM meets many of them.
fn drawn_source(tc: &TestCase) -> String {
    let opening: &str = tc.draw(gs::sampled_from(&OPENINGS));
    let pieces: Vec<&str> = tc.draw(gs::vecs(gs::sampled_from(&PIECES)).min_size(8).max_size(48));
    format!("{opening}{}", pieces.concat())
}

/// What the Bux parser writes for each source, in order, and nothing where no JDK can run it.
fn printed_by_the_bux_parser(sources: &[String]) -> Option<Vec<String>> {
    if jdk().is_none() {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the Bux parser needs one");
        return None;
    }
    let program = the_bux_parser_beside(WRITES_THE_ANSWER);
    Some(answers_of_one_run(&program, sources))
}

/// A program of `content`, with each module of the Bux parser beside it.
fn the_bux_parser_beside(content: &str) -> Example {
    let program = Example::new(content);
    for module in MODULES {
        let source = fs::read_to_string(compiler_module(module)).expect("a module is readable");
        program.beside_it(&Sibling {
            named: module,
            content: &source,
        });
    }
    program
}

/// The Rust parser's answer: the printed tree, or the printed error where parsing fails.
fn answer(source: &str) -> String {
    match lumen_parser::parse(source) {
        Ok(_) => printed::render(source),
        Err(_) => printed::render_error(source),
    }
}

/// Every `.lm` example under `tests/spec/parser`, in the order their names sort.
fn parser_examples() -> Vec<PathBuf> {
    let mut examples: Vec<PathBuf> = fs::read_dir(repository().join("tests/spec/parser"))
        .expect("tests/spec/parser exists")
        .map(|entry| entry.expect("a directory entry is readable").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "lm"))
        .collect();
    examples.sort();
    examples
}

fn compiler_module(module: &str) -> PathBuf {
    repository().join(format!("compiler/{module}.lm"))
}

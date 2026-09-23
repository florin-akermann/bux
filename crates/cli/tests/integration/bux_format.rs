//! The formatter written in Bux, `compiler/format.lm`, held to the Rust formatter's text.
//!
//! `docs/specs/formatting.md` states the behaviour, and the Rust formatter is the answer the Bux
//! one must give. Building the Bux formatter needs no JDK; running what was built needs one, and a
//! test that runs it is skipped with a named reason when `JAVA_HOME` names none.

use std::fs;
use std::path::{Path, PathBuf};

use hegel::TestCase;
use hegel::generators as gs;

use crate::common::{Example, Sibling, Within, as_argument, jdk, lumen, repository};

/// A program that writes, beside each file it is run with, what the Bux formatter says of it.
///
/// One run formats every file, because each run is one compiler and one JVM.
const WRITES_THE_ANSWER: &str = "import files\n\nimport format\n\nfn main(arguments: List<String>) -> Int {\n    for path in arguments {\n        _ = files.write(path + \".answer\", answer(ok_or(files.read(path), \"\")))\n    }\n    0\n}\n\n/// What the Bux formatter says of `source`: the canonical text, or that it refuses it.\n// example: answer(\"fn (\") == \"refused\\n\"\nfn answer(source: String) -> String {\n    match format.format(source) {\n        Ok(canonical) => \"canonical\\n\" + canonical\n        Err(refused) => \"refused\\n\"\n    }\n}\n";

/// The modules of the Bux compiler that the formatter is, beside the program as siblings.
const MODULES: [&str; 4] = ["lexer", "ast", "parser", "format"];

/// How many runs the property makes, which is one compiler and one JVM each.
const CASES: u64 = 10;

/// What a drawn source is made of: items written out of canonical form, most of which parse.
const ITEMS: [&str; 12] = [
    "import  io",
    "// a comment alone   ",
    "type   Id = Id(Int)",
    "type Shape = | Dot // a dot\n| Line { length: Int\nwidth:Int }",
    "type Empty = {}",
    "extern static read(int at: Int,path:Path)->String = \"a.B.c\"",
    "derive Eq ,Hash for Id",
    "trait Named<T> {\n// the name\nfn name(value: T)->String\n}",
    "fn main( ) -> Int {\n  var total = 0 // start\n   total += -(7.abs())\n\n  total // done\n}",
    "fn pick(x: Int) -> String {\n match x {\n 1 | 2 => \"few\" // small\n _ => if x > 9 { \"many\" } else if (User {}).on { \"on\" } else { \"some\" }\n }\n}",
    "fn go<T:Eq<T>>(users: List<T>) {\nfor user in users { _ = f(a: (1 + 2) * 3, b: !(a && b)) }\nfor { break }\n}",
    "fn broken( {",
];

/// What is written between two drawn items.
const BETWEEN: [&str; 5] = ["\n", "\n\n", "\n\n\n", "\n// between\n", "   \n"];

#[test]
fn the_bux_formatter_builds_under_the_rust_compiler() {
    let program = the_bux_formatter_beside(WRITES_THE_ANSWER);

    let run = lumen(&["build", as_argument(&program.path)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn every_example_the_bux_formatter_states_holds() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the examples needs one");
        return;
    };

    let run = lumen(&[
        "test",
        as_argument(&repository().join("compiler/format.lm")),
    ]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

/// `docs/specs/formatting.md`: a file in canonical form formats to itself, byte for byte.
#[test]
fn the_bux_formatter_leaves_every_lm_file_of_the_repository_as_it_is() {
    let files = lm_files_under(&repository());
    assert!(
        files
            .iter()
            .any(|file| file.ends_with("compiler/format.lm")),
        "the walk reaches compiler/"
    );
    let sources: Vec<String> = files.iter().map(|file| read(file)).collect();

    let Some(answers) = answers_of_the_bux_formatter(&sources) else {
        return;
    };

    for ((file, source), answer) in files.iter().zip(&sources).zip(answers) {
        let wanted = match lumen_format::format(source) {
            Ok(_) => format!("canonical\n{source}"),
            Err(_) => "refused\n".to_owned(),
        };
        assert_eq!(answer, wanted, "{}", file.display());
    }
}

/// `docs/specs/formatting.md`: every format example, canonical or not, formats as Rust formats it.
#[test]
fn the_bux_formatter_gives_the_rust_formatters_text_on_every_format_example() {
    let examples = format_examples();
    assert!(
        examples
            .iter()
            .any(|example| example.extension().is_some_and(|it| it == "unformatted")),
        "tests/spec/format holds an example not in canonical form"
    );
    let sources: Vec<String> = examples.iter().map(|example| read(example)).collect();

    let Some(answers) = answers_of_the_bux_formatter(&sources) else {
        return;
    };

    for ((example, source), answer) in examples.iter().zip(&sources).zip(answers) {
        assert_eq!(answer, rust_answer(source), "{}", example.display());
    }
}

/// `docs/specs/formatting.md`: the Bux formatter and the Rust formatter give one text on any file.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn the_bux_formatter_gives_the_rust_formatters_text_on_drawn_files(tc: TestCase) {
    let sources: Vec<String> = (0..8).map(|_| drawn_source(&tc)).collect();

    let Some(answers) = answers_of_the_bux_formatter(&sources) else {
        return;
    };

    for (source, answer) in sources.iter().zip(answers) {
        assert_eq!(answer, rust_answer(source), "{source:?}");
    }
}

/// Items and what separates them, run together as one file.
fn drawn_source(tc: &TestCase) -> String {
    let items: Vec<&str> = tc.draw(gs::vecs(gs::sampled_from(&ITEMS)).min_size(1).max_size(6));
    let mut source = String::new();
    for item in items {
        source.push_str(item);
        source.push_str(tc.draw(gs::sampled_from(&BETWEEN)));
    }
    source
}

/// What the Rust formatter says of `source`, in the form the Bux program writes its answer.
fn rust_answer(source: &str) -> String {
    match lumen_format::format(source) {
        Ok(canonical) => format!("canonical\n{canonical}"),
        Err(_) => "refused\n".to_owned(),
    }
}

/// What the Bux formatter says of each source, in order, and nothing where no JDK can run it.
fn answers_of_the_bux_formatter(sources: &[String]) -> Option<Vec<String>> {
    if jdk().is_none() {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the Bux formatter needs one");
        return None;
    }
    let program = the_bux_formatter_beside(WRITES_THE_ANSWER);
    let inputs: Vec<PathBuf> = sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let at = format!("inputs/{index}.txt");
            program.within_it(&Within {
                at: &at,
                content: source,
            });
            program.directory.join(at)
        })
        .collect();
    let mut arguments = vec!["run", as_argument(&program.path)];
    arguments.extend(inputs.iter().map(|input| as_argument(input)));

    let run = lumen(&arguments);

    assert_eq!(run.code, 0, "{}", run.stderr);
    Some(
        inputs
            .iter()
            .map(|input| read(&input.with_extension("txt.answer")))
            .collect(),
    )
}

/// A program of `content`, with each module of the Bux formatter beside it.
fn the_bux_formatter_beside(content: &str) -> Example {
    let program = Example::new(content);
    for module in MODULES {
        let source = read(&repository().join(format!("compiler/{module}.lm")));
        program.beside_it(&Sibling {
            named: module,
            content: &source,
        });
    }
    program
}

/// Every `.lm` file under `directory`, leaving out build output and hidden directories.
///
/// A hidden directory holds tools' state, such as a worktree of another checkout, and none of it
/// is source of this repository.
fn lm_files_under(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(directory).expect("a directory of the repository is readable") {
        let path = entry.expect("a directory entry is readable").path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        if path.is_dir() {
            found.extend(lm_files_under(&path));
        } else if path.extension().is_some_and(|extension| extension == "lm") {
            found.push(path);
        }
    }
    found.sort();
    found
}

/// Every example under `tests/spec/format`, canonical or not, in the order their names sort.
fn format_examples() -> Vec<PathBuf> {
    let mut examples: Vec<PathBuf> = fs::read_dir(repository().join("tests/spec/format"))
        .expect("tests/spec/format exists")
        .map(|entry| entry.expect("a directory entry is readable").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "lm" || extension == "unformatted")
        })
        .collect();
    examples.sort();
    examples
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("a file of the repository is UTF-8")
}

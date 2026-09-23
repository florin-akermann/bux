//! Type inference written in Bux, `compiler/types.lm`, held to the Rust phase's answer.
//!
//! `docs/specs/types.md`, `docs/specs/traits.md`, and `docs/specs/derive.md` state the behaviour,
//! and the Rust phase is the answer the Bux one must give: the same API page of the last module,
//! or the same refusal. Building the Bux phase needs no JDK; running what was built needs one, and
//! a test that runs it is skipped with a named reason when `JAVA_HOME` names none.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use hegel::TestCase;
use hegel::generators as gs;
use lumen_modules::load;
use lumen_types::Imported;

use crate::common::{Example, Sibling, Within, as_argument, jdk, lumen, repository};

/// The modules of the Bux compiler that the type inference is, beside the program as siblings.
const MODULES: [&str; 13] = [
    "lexer", "ast", "parser", "format", "modules", "resolver", "unify", "refusal", "boundary",
    "surface", "declared", "infer", "types",
];

/// The modules whose examples this harness runs, which are the ones this phase adds.
const PHASE: [&str; 7] = [
    "unify", "refusal", "boundary", "surface", "declared", "infer", "types",
];

/// The directories whose every `.lm` file is a fixture, the Bux compiler's own modules among them.
const FIXTURES: [&str; 3] = ["tests/spec", "library", "compiler"];

/// How many runs the property makes, which is one compiler and one JVM each.
const CASES: u64 = 5;

/// How many drawn modules one run checks.
const MODULES_A_RUN: usize = 24;

#[test]
fn the_bux_type_inference_builds_under_the_rust_compiler() {
    let program = the_bux_phase_beside(&writes_the_answer());

    let run = lumen(&["build", as_argument(&program.path)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn every_example_the_bux_type_inference_states_holds() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the examples needs one");
        return;
    };

    for module in PHASE {
        let run = lumen(&[
            "test",
            as_argument(&repository().join(format!("compiler/{module}.lm"))),
        ]);

        assert_eq!(run.code, 0, "{module}: {}", run.stderr);
    }
}

/// `docs/specs/types.md`: on every fixture, the Bux phase gives the Rust phase's answer.
#[test]
fn the_bux_type_inference_gives_the_rust_answer_on_every_fixture() {
    let program = the_bux_phase_beside(&writes_the_answer());
    let fixtures = fixtures();

    let Some(answers) = answers_of_the_bux_phase(&program, &fixtures) else {
        return;
    };

    for (fixture, answer) in fixtures.iter().zip(answers) {
        assert_eq!(answer, rust_answer(fixture), "{}", fixture.display());
    }
}

/// `docs/specs/types.md`: on a drawn module, the two phases give one answer.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn the_bux_type_inference_gives_the_rust_answer_on_drawn_modules(tc: TestCase) {
    let program = the_bux_phase_beside(&writes_the_answer());
    let roots: Vec<PathBuf> = (0..MODULES_A_RUN)
        .map(|index| {
            let at = format!("drawn/{index}/main.lm");
            program.within_it(&Within {
                at: &at,
                content: &drawn_module(&tc),
            });
            program.directory.join(at)
        })
        .collect();

    let Some(answers) = answers_of_the_bux_phase(&program, &roots) else {
        return;
    };

    for (root, answer) in roots.iter().zip(answers) {
        let source = fs::read_to_string(root).expect("a drawn module is UTF-8");
        assert_eq!(answer, rust_answer(root), "{source}");
    }
}

/// The types a drawn signature writes.
const TYPES: [&str; 6] = ["Int", "Bool", "String", "Option<Int>", "List<Int>", "Box"];

/// The expressions a drawn body is made of, over the parameters `one` and `other`.
const EXPRESSIONS: [&str; 16] = [
    "one",
    "other",
    "1",
    "\"text\"",
    "one + other",
    "one == other",
    "one < 2",
    "one / 0",
    "one / other",
    "Some(one)",
    "[one, other]",
    "one?",
    "Box { held: one }",
    "one.held",
    "pick(one, other)",
    "!one",
];

/// A module of one record and two functions, the second of which the first calls.
fn drawn_module(tc: &TestCase) -> String {
    let body = drawn_body(tc);
    format!(
        "fn check({}, {}) -> {} {{\n{body}}}\n\nfn pick<T>(first: T, second: T) -> T {{\n    first\n}}\n\ntype Box = {{\n    held: Int\n}}\n",
        drawn_parameter(tc, "one"),
        drawn_parameter(tc, "other"),
        pick(tc, &TYPES)
    )
}

/// A parameter called `named`, whose type is left for inference to find where the draw says so.
fn drawn_parameter(tc: &TestCase, named: &str) -> String {
    if tc.draw(gs::booleans()) {
        return named.to_owned();
    }
    format!("{named}: {}", pick(tc, &TYPES))
}

/// A body of up to two bindings and a drawn expression, or a `match` over a drawn one.
fn drawn_body(tc: &TestCase) -> String {
    let mut body = String::new();
    let bindings: usize = tc.draw(gs::integers().min_value(0).max_value(2));
    for index in 0..bindings {
        let _ = writeln!(body, "    held{index} := {}", pick(tc, &EXPRESSIONS));
    }
    if tc.draw(gs::booleans()) {
        let _ = write!(
            body,
            "    match {} {{\n        1 => {}\n        _ => {}\n    }}\n",
            pick(tc, &EXPRESSIONS),
            pick(tc, &EXPRESSIONS),
            pick(tc, &EXPRESSIONS)
        );
    } else {
        let _ = writeln!(body, "    {}", pick(tc, &EXPRESSIONS));
    }
    body
}

fn pick(tc: &TestCase, among: &[&'static str]) -> &'static str {
    tc.draw(gs::sampled_from(among))
}

/// What the Rust phase says of the program at `path`, in the form the Bux phase prints its own.
///
/// A program that does not load or resolve is `skipped`, because that is a phase before this one.
fn rust_answer(path: &Path) -> String {
    let Ok(loaded) = load(path) else {
        return "skipped\n".to_owned();
    };
    let mut imported = Imported::default();
    let mut page = String::new();
    for module in loaded.into_modules() {
        let Ok(resolved) = lumen_resolver::resolve(module.program().clone(), module.name()) else {
            return "skipped\n".to_owned();
        };
        match lumen_types::check(resolved, &imported) {
            Ok(typed) => {
                page = lumen_api::surface(&typed);
                imported = imported.offering(module.name(), typed.surface().clone());
            }
            Err(refused) => return refusal_printed(module.name(), &refused),
        }
    }
    format!("typed\n{page}")
}

/// A refusal of the module called `module`, as `docs/specs/diagnostics.md` states its parts.
fn refusal_printed(module: &str, refused: &lumen_types::TypeError) -> String {
    let said = refused.diagnostic();
    let span = said.span();
    format!(
        "refused {module}\n{} {}..{} {}\nhelp: {}\n",
        said.code().number(),
        span.start(),
        span.end(),
        said.message(),
        said.help().unwrap_or("")
    )
}

/// A program that writes the Bux phase's answer for each path after the first, which names
/// the directory the answers go to.
///
/// One run checks every program, because each run is one compiler and one JVM. An answer is
/// written under the program's own directory, so nothing is written inside the repository.
fn writes_the_answer() -> String {
    let prelude = as_argument(&prelude_path())
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    format!(
        "import files\n\nimport list\n\nimport types\n\nfn main(arguments: List<String>) -> Int {{\n    prelude := types.prelude_read(ok_or(files.read(\"{prelude}\"), \"\"))\n    answers := or(list.at(arguments, 0), \".\")\n    var index = 0\n    for path in arguments {{\n        if index > 0 {{\n            _ = files.write(answers + \"/\" + shown(index) + \".answer\", types.printed(path, prelude))\n        }}\n        index += 1\n    }}\n    0\n}}\n"
    )
}

/// What the Bux phase says of each root, in order, and nothing where no JDK can run it.
///
/// The program is built with `lumen` and started on a JVM whose class path holds the repository
/// as well as the classes, which is what puts `library/` where the Bux loader asks for it.
fn answers_of_the_bux_phase(program: &Example, roots: &[PathBuf]) -> Option<Vec<String>> {
    let Some(java) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the Bux type inference needs one");
        return None;
    };
    let built = lumen(&["build", as_argument(&program.path)]);
    assert_eq!(built.code, 0, "{}", built.stderr);
    let answers = program.directory.join("answers");
    fs::create_dir_all(&answers).expect("the answers' directory is creatable");
    let class_path = std::env::join_paths([program.directory.clone(), repository()])
        .expect("a class path joins");

    let ran = Command::new(java)
        .arg("--enable-preview")
        .arg("-cp")
        .arg(class_path)
        .arg("example")
        .arg(&answers)
        .args(roots)
        .output()
        .expect("the JVM starts");

    assert!(
        ran.status.success(),
        "{}",
        String::from_utf8_lossy(&ran.stderr)
    );
    Some(
        (1..=roots.len())
            .map(|index| {
                fs::read_to_string(answers.join(format!("{index}.answer")))
                    .expect("the program writes a UTF-8 answer for each root")
            })
            .collect(),
    )
}

/// A program of `content`, with each module of the Bux type inference beside it.
fn the_bux_phase_beside(content: &str) -> Example {
    let program = Example::new(content);
    for module in MODULES {
        let source = fs::read_to_string(repository().join(format!("compiler/{module}.lm")))
            .expect("a module is readable");
        program.beside_it(&Sibling {
            named: module,
            content: &source,
        });
    }
    program
}

/// Every `.lm` file under the fixture directories, in the order their paths sort.
fn fixtures() -> Vec<PathBuf> {
    let mut found = Vec::new();
    for directory in FIXTURES {
        gather(&repository().join(directory), &mut found);
    }
    found.sort();
    found
}

/// Every `.lm` file under `directory`, however deep.
fn gather(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("a fixture directory is readable") {
        let path = entry.expect("a directory entry is readable").path();
        if path.is_dir() {
            gather(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "lm") {
            found.push(path);
        }
    }
}

fn prelude_path() -> PathBuf {
    repository()
        .join("library/prelude.lm")
        .canonicalize()
        .expect("the prelude is there")
}

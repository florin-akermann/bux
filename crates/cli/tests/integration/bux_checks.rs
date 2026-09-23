//! The two checks after inference written in Bux, held to the answers of the Rust phases.
//!
//! `compiler/exhaustiveness.lm` is the exhaustiveness check, and `compiler/holes.lm` lists the
//! holes. `docs/specs/exhaustiveness.md` and `docs/specs/holes.md` state the behaviour, and the
//! Rust phases give the answer the Bux ones must give: the same refusal, or the same holes. Building
//! the Bux phases needs no JDK; running them needs one, and a test that runs them is skipped with a
//! named reason when `JAVA_HOME` names none.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use hegel::TestCase;
use hegel::generators as gs;
use lumen_diagnostics::Diagnostic;
use lumen_holes::Whole;
use lumen_modules::load;
use lumen_types::{Imported, TypedProgram};

use crate::common::{Example, Sibling, Within, as_argument, jdk, lumen, repository};

/// The modules of the Bux compiler the two checks read, beside the program as siblings.
const MODULES: [&str; 15] = [
    "lexer",
    "ast",
    "parser",
    "format",
    "modules",
    "resolver",
    "unify",
    "refusal",
    "boundary",
    "surface",
    "declared",
    "infer",
    "types",
    "exhaustiveness",
    "holes",
];

/// The modules whose examples this harness runs, which are the ones this phase adds.
const PHASE: [&str; 2] = ["exhaustiveness", "holes"];

/// The directories whose every `.lm` file is a fixture, the Bux compiler's own modules among them.
const FIXTURES: [&str; 3] = ["tests/spec", "library", "compiler"];

/// How many runs a property makes, which is one compiler and one JVM each.
const CASES: u64 = 5;

/// How many drawn modules one run checks.
const MODULES_A_RUN: usize = 24;

/// The two answers each run writes for a program, one file each.
#[derive(Clone, Copy)]
enum Check {
    /// `exhaustiveness.printed`: `exhaustive`, or the first `match` refused.
    Exhaustiveness,
    /// `holes.printed`: `checked` and every hole, or `skipped`.
    Holes,
}

impl Check {
    /// The extension of the file a run writes this answer to.
    const fn extension(self) -> &'static str {
        match self {
            Self::Exhaustiveness => "exhaustiveness",
            Self::Holes => "holes",
        }
    }

    /// What the Rust phases say of the program at `path`, in the form the Bux phase prints.
    fn rust_answer(self, path: &Path) -> String {
        let checked = checked_by_rust(path);
        match self {
            Self::Exhaustiveness => {
                checked.map_or_else(|refused| refused, |_| "exhaustive\n".to_owned())
            }
            Self::Holes => {
                checked.map_or_else(|_| "skipped\n".to_owned(), |typed| holes_listed(&typed))
            }
        }
    }
}

#[test]
fn the_bux_checks_build_under_the_rust_compiler() {
    let program = the_bux_checks_beside(&writes_the_answers());

    let run = lumen(&["build", as_argument(&program.path)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn every_example_the_bux_checks_state_holds() {
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

/// `docs/specs/exhaustiveness.md`: on every fixture, the Bux check gives the Rust answer.
#[test]
fn the_bux_exhaustiveness_check_gives_the_rust_answer_on_every_fixture() {
    every_fixture_gives_the_rust_answer(Check::Exhaustiveness);
}

/// `docs/specs/holes.md`: on every fixture, the Bux listing names the holes the Rust phase names.
#[test]
fn the_bux_hole_listing_gives_the_rust_listing_on_every_fixture() {
    every_fixture_gives_the_rust_answer(Check::Holes);
}

/// `docs/specs/exhaustiveness.md`: on a drawn `match`, the two checks give one answer.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn the_bux_exhaustiveness_check_gives_the_rust_answer_on_drawn_matches(tc: TestCase) {
    drawn_modules_give_the_rust_answer(&tc, drawn_match, Check::Exhaustiveness);
}

/// `docs/specs/holes.md`: on a drawn body of holes, the two listings name the same holes.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn the_bux_hole_listing_gives_the_rust_listing_on_drawn_holes(tc: TestCase) {
    drawn_modules_give_the_rust_answer(&tc, drawn_holes, Check::Holes);
}

fn every_fixture_gives_the_rust_answer(check: Check) {
    let program = the_bux_checks_beside(&writes_the_answers());
    let fixtures = fixtures();

    let Some(answers) = answers_of_the_bux_phase(&program, &fixtures, check) else {
        return;
    };

    for (fixture, answer) in fixtures.iter().zip(answers) {
        assert_eq!(answer, check.rust_answer(fixture), "{}", fixture.display());
    }
}

fn drawn_modules_give_the_rust_answer(tc: &TestCase, draw: fn(&TestCase) -> String, check: Check) {
    let program = the_bux_checks_beside(&writes_the_answers());
    let roots: Vec<PathBuf> = (0..MODULES_A_RUN)
        .map(|index| {
            let at = format!("drawn/{index}/main.lm");
            program.within_it(&Within {
                at: &at,
                content: &draw(tc),
            });
            program.directory.join(at)
        })
        .collect();

    let Some(answers) = answers_of_the_bux_phase(&program, &roots, check) else {
        return;
    };

    for (root, answer) in roots.iter().zip(answers) {
        let source = fs::read_to_string(root).expect("a drawn module is UTF-8");
        assert_eq!(answer, check.rust_answer(root), "{source}");
    }
}

/// The patterns an arm over each parameter of a drawn `match` writes, by the parameter's name.
const ARMS: [(&str, &[&str]); 4] = [
    (
        "status",
        &[
            "Pending",
            "Running(_)",
            "Running(1)",
            "Failed(_, true)",
            "Failed(0, _)",
            "Failed(_, _)",
            "Pending | Running(_)",
            "Running(_) | Pending",
            "Failed(_, false) | Pending",
            "other",
            "_",
        ],
    ),
    (
        "maybe",
        &[
            "Some(Pending)",
            "Some(Running(_))",
            "Some(Failed(_, _))",
            "Some(Pending | Failed(_, _))",
            "Some(Failed(_, _) | Running(2))",
            "None",
            "Some(other)",
            "Some(_) | None",
            "None | Some(_)",
            "_",
        ],
    ),
    ("count", &["0", "1 | 2", "other", "_"]),
    ("flag", &["true", "false", "false | true", "_"]),
];

/// A module of one `match` of up to four drawn arms over one of four parameters.
fn drawn_match(tc: &TestCase) -> String {
    let (scrutinee, patterns) = tc.draw(gs::sampled_from(&ARMS));
    let arms: usize = tc.draw(gs::integers().min_value(1).max_value(4));
    let mut body = String::new();
    for _ in 0..arms {
        let _ = writeln!(body, "        {} => 1", pick(tc, patterns));
    }
    format!(
        "fn check(status: Status, maybe: Option<Status>, count: Int, flag: Bool) -> Int {{\n    match {scrutinee} {{\n{body}    }}\n}}\n\ntype Status =\n    | Pending\n    | Running(Int)\n    | Failed(Int, Bool)\n"
    )
}

/// The statements a drawn body of holes writes before its last expression.
const STATEMENTS: [&str; 6] = [
    "_ = todo(\"a\")",
    "_ = count + todo(\"b\")",
    "_ = [todo(\"c\"), count]",
    "_ = todo(todo(\"d\"))",
    "for todo(\"e\") {\n        break\n    }",
    "_ = pick(first: count, second: todo(\"f\"))",
];

/// The last expression of a drawn body of holes, which gives the function's value.
const LAST: [&str; 6] = [
    "todo(\"z\")",
    "count",
    "pick(todo(\"x\"), todo(\"y\"))",
    "if flag {\n        todo(\"i\")\n    } else {\n        count\n    }",
    "match count {\n        1 => todo(\"m\")\n        _ => count\n    }",
    "Box { held: todo(\"r\") }.held",
];

/// A module whose first function writes up to three drawn statements and a drawn last expression.
fn drawn_holes(tc: &TestCase) -> String {
    let statements: usize = tc.draw(gs::integers().min_value(0).max_value(3));
    let mut body = String::new();
    for _ in 0..statements {
        let _ = writeln!(body, "    {}", pick(tc, &STATEMENTS));
    }
    let _ = writeln!(body, "    {}", pick(tc, &LAST));
    format!(
        "fn check(flag: Bool, count: Int) -> Int {{\n{body}}}\n\nfn pick(first: Int, second: Int) -> Int {{\n    first\n}}\n\ntype Box = {{\n    held: Int\n}}\n"
    )
}

fn pick(tc: &TestCase, among: &[&'static str]) -> &'static str {
    tc.draw(gs::sampled_from(among))
}

/// Each module of the program at `path`, typed and checked, or the answer where one is not.
///
/// A program that does not load, resolve, or type is `skipped`, because that is a phase before
/// these two. A `match` the Rust phase refuses is the refusal, in the form the Bux phase prints.
fn checked_by_rust(path: &Path) -> Result<Vec<(String, TypedProgram)>, String> {
    let skipped = || "skipped\n".to_owned();
    let loaded = load(path).map_err(|_| skipped())?;
    let mut imported = Imported::default();
    let mut checked = Vec::new();
    for module in loaded.into_modules() {
        let resolved = lumen_resolver::resolve(module.program().clone(), module.name())
            .map_err(|_| skipped())?;
        let typed = lumen_types::check(resolved, &imported).map_err(|_| skipped())?;
        lumen_exhaustiveness::check(&typed).map_err(|refused| {
            format!(
                "refused {}\n{}",
                module.name(),
                printed(&refused.diagnostic())
            )
        })?;
        imported = imported.offering(module.name(), typed.surface().clone());
        checked.push((module.name().to_owned(), typed));
    }
    Ok(checked)
}

/// `checked`, then every hole of each module, each named with the module it is in.
fn holes_listed(typed: &[(String, TypedProgram)]) -> String {
    let mut listed = "checked\n".to_owned();
    for (module, program) in typed {
        for hole in Whole::of_module(program).err().unwrap_or_default() {
            let _ = write!(listed, "{module} {}", printed(&hole.diagnostic()));
        }
    }
    listed
}

/// A diagnostic as `docs/specs/diagnostics.md` states its parts: the code, the span, and the
/// message, then the help.
fn printed(said: &Diagnostic) -> String {
    let span = said.span();
    format!(
        "{} {}..{} {}\nhelp: {}\n",
        said.code().number(),
        span.start(),
        span.end(),
        said.message(),
        said.help().unwrap_or("")
    )
}

/// A program that writes both answers of the Bux phases for each path after the first, which
/// names the directory the answers go to.
///
/// One run checks every program, because each run is one compiler and one JVM. An answer is
/// written under the program's own directory, so nothing is written inside the repository.
fn writes_the_answers() -> String {
    let prelude = as_argument(&prelude_path())
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    format!(
        "import exhaustiveness\n\nimport files\n\nimport holes\n\nimport list\n\nimport types\n\nfn main(arguments: List<String>) -> Int {{\n    prelude := types.prelude_read(ok_or(files.read(\"{prelude}\"), \"\"))\n    answers := or(list.at(arguments, 0), \".\")\n    var index = 0\n    for path in arguments {{\n        if index > 0 {{\n            _ = files.write(answers + \"/\" + shown(index) + \".exhaustiveness\", exhaustiveness.printed(path, prelude))\n            _ = files.write(answers + \"/\" + shown(index) + \".holes\", holes.printed(path, prelude))\n        }}\n        index += 1\n    }}\n    0\n}}\n"
    )
}

/// What the Bux phase says of each root, in order, and nothing where no JDK can run it.
///
/// The program is built with `lumen` and started on a JVM whose class path holds the repository
/// as well as the classes, which is what puts `library/` where the Bux loader asks for it.
fn answers_of_the_bux_phase(
    program: &Example,
    roots: &[PathBuf],
    check: Check,
) -> Option<Vec<String>> {
    let Some(java) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the Bux checks needs one");
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
                fs::read_to_string(answers.join(format!("{index}.{}", check.extension())))
                    .expect("the program writes a UTF-8 answer for each root")
            })
            .collect(),
    )
}

/// A program of `content`, with each module of the two Bux checks beside it.
fn the_bux_checks_beside(content: &str) -> Example {
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

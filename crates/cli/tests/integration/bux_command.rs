//! The command line written in Bux, `compiler/main.lm` over `compiler/command.lm`, held to `lumen`.
//!
//! `docs/specs/run.md` states the behaviour: for every command and every fixture, the Bux command
//! line writes what the Rust binary writes on each stream and ends with the same status, byte for
//! byte, and it writes the same files. Building it needs no JDK; running it needs one, and a test
//! that runs it is skipped with a named reason when `JAVA_HOME` names none.
//!
//! One JVM answers a whole list of command lines through the driver `answers.lm`, which is the
//! harness shape of every `bux_*` module; `common.rs` holds the stage it works on.

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::common::{
    BUX, Case, DRIVEN, FIXTURES, Home, RUST, SHARDS, Stage, as_argument, fixture_cases, jdk, lumen,
    repository, the_same_from_the_driver,
};

/// The command lines the argument parser is held to, one a line; the blank line gives no words.
const COMMAND_LINES: &str = include_str!("command_lines.txt");

#[test]
fn every_example_the_bux_command_line_states_holds() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the examples needs one");
        return;
    };

    let run = lumen(&[
        "test",
        as_argument(&repository().join("compiler/command.lm")),
    ]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

/// `docs/specs/run.md`: every command line gets the answer `lumen` gives, whatever words it has.
#[test]
fn the_bux_command_line_answers_every_command_line_as_lumen_does() {
    let cases: Vec<Case> = COMMAND_LINES.lines().map(Case::written).collect();

    the_same_from_the_driver(&cases, "answering a command line needs one");
}

/// `docs/specs/diagnostics.md`: every fixture is checked, in both forms, as `lumen` checks it.
#[test]
fn the_bux_command_line_checks_every_fixture_as_lumen_does() {
    let cases = fixture_cases(&["check", "check --json"], &FIXTURES);

    the_same_from_the_driver(&cases, "checking a fixture needs one");
}

/// `docs/specs/surface.md`: the surface of every fixture is printed as `lumen` prints it.
#[test]
fn the_bux_command_line_prints_the_surface_of_every_fixture_as_lumen_does() {
    let cases = fixture_cases(&["api"], &FIXTURES);

    the_same_from_the_driver(&cases, "printing a surface needs one");
}

/// `docs/specs/formatting.md` and `docs/specs/codegen.md`: every fixture of `tests/spec` and
/// `library` is formatted and built as `lumen` formats and builds it, down to each byte written.
#[test]
fn every_fixture_under_tests_spec_and_library_is_formatted_and_built_as_lumen_does_it() {
    the_same_files_written(&["tests/spec", "library"]);
}

/// The same, for every module of the compiler.
#[test]
fn every_module_of_the_compiler_is_formatted_and_built_as_lumen_does_it() {
    the_same_files_written(&["compiler"]);
}

/// The same answers from `lumen` and from the driver to `fmt` and `build` on every fixture under
/// `directories`, and the same files on each side afterwards.
fn the_same_files_written(directories: &[&str]) {
    let cases = fixture_cases(&["fmt", "build"], directories);

    let Some(stage) = the_same_from_the_driver(&cases, "formatting a fixture needs one") else {
        return;
    };

    assert_eq!(
        files_under(&stage.root.join(BUX)),
        files_under(&stage.root.join(RUST))
    );
}

/// `docs/specs/diagnostics.md`: every code is explained as `lumen` explains it, and every path
/// that is a package, a directory, or no readable file is answered as `lumen` answers it.
#[test]
fn the_bux_command_line_explains_every_code_and_reads_every_path_as_lumen_does() {
    let mut cases = explanation_cases();
    cases.extend(directory_cases());
    cases.extend(unreadable_cases());

    the_same_from_the_driver(&cases, "explaining a code needs one");
}

/// `docs/specs/packages.md`: a package whose directory cannot be listed is refused as `lumen`
/// refuses it, and never passes as a package with no module.
#[test]
fn the_bux_command_line_refuses_a_package_it_cannot_list_as_lumen_does() {
    let Some(java) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and starting the Bux command line needs one");
        return;
    };
    let stage = Stage::built(DRIVEN);
    let cases = [
        Case::written("check unlisted"),
        Case::written("build unlisted"),
    ];

    let packages = unlisted_packages(&stage);
    moded(&packages, 0o311);
    let rust: Vec<_> = cases
        .iter()
        .map(|case| stage.said_by_rust(case, 0, Home::Inherited))
        .collect();
    let bux = stage.said_by_the_driver(&java, &cases, Home::Inherited);
    moded(&packages, 0o755);

    assert_eq!(bux, rust);
    assert_eq!(rust[0].status, 2, "{}", rust[0].errors);
}

/// The package `unlisted`, a manifest and one module, made in every copy of the stage.
fn unlisted_packages(stage: &Stage) -> Vec<PathBuf> {
    let mut packages = Vec::new();
    for side in [RUST, BUX] {
        for shard in 0..SHARDS {
            let package = stage.side(side, shard).join("unlisted");
            fs::create_dir_all(&package).expect("a package directory is creatable");
            fs::write(package.join("bux.package"), "").expect("a manifest is writable");
            fs::write(
                package.join("a.lm"),
                "fn main(arguments: List<String>) -> Int {\n    0\n}\n",
            )
            .expect("a module is writable");
            packages.push(package);
        }
    }
    packages
}

/// Sets `mode` on every directory of `packages`.
///
/// Mode `0o311` lets a manifest be found but no directory be listed; `0o755` undoes it, so the
/// stage can be removed.
fn moded(packages: &[PathBuf], mode: u32) {
    for package in packages {
        fs::set_permissions(package, fs::Permissions::from_mode(mode))
            .expect("the mode of a package directory is settable");
    }
}

/// `docs/specs/run.md`: a run that finds no JDK is refused in the words `lumen` refuses it with.
#[test]
fn the_bux_command_line_refuses_a_run_without_a_jdk_as_lumen_does() {
    let Some(java) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and starting the Bux command line needs one");
        return;
    };
    let stage = Stage::built(DRIVEN);
    let case = Case::written("run echo.lm a");

    for home in [Home::Unset, Home::At("/no/such/jdk")] {
        let rust = stage.said_by_rust(&case, 0, home);
        let bux = stage.said_by_the_driver(&java, std::slice::from_ref(&case), home);

        assert_eq!(bux, vec![rust], "JAVA_HOME {home:?}");
    }
}

/// `docs/specs/doc-examples.md`: every fixture of `tests/spec` and `library` is tested as `lumen`
/// tests it.
#[test]
fn every_fixture_under_tests_spec_and_library_is_tested_as_lumen_tests_it() {
    let cases = fixture_cases(&["test"], &["tests/spec", "library"]);

    the_same_from_the_driver(&cases, "testing an example needs one");
}

/// `docs/specs/doc-examples.md`: the first half of the compiler's modules is tested as `lumen`
/// tests it. Each module's examples start a JVM on both sides, so the compiler is halved.
#[test]
fn the_first_half_of_the_compiler_is_tested_as_lumen_tests_it() {
    the_same_from_the_driver(
        &compiler_half(Half::First),
        "testing the compiler needs one",
    );
}

/// `docs/specs/doc-examples.md`: the second half of the compiler's modules, likewise.
#[test]
fn the_second_half_of_the_compiler_is_tested_as_lumen_tests_it() {
    the_same_from_the_driver(
        &compiler_half(Half::Second),
        "testing the compiler needs one",
    );
}

/// One half of the compiler's modules, by the order their paths sort.
#[derive(Clone, Copy)]
enum Half {
    First,
    Second,
}

/// `test` on one half of the compiler's modules.
fn compiler_half(half: Half) -> Vec<Case> {
    let mut all = fixture_cases(&["test"], &["compiler"]);
    let second = all.split_off(all.len() / 2);
    match half {
        Half::First => all,
        Half::Second => second,
    }
}

/// `explain` on every code there is an explanation of, and on words that name none.
fn explanation_cases() -> Vec<Case> {
    let mut codes: Vec<String> = fs::read_dir(repository().join("compiler/explanations"))
        .expect("the explanations are readable")
        .map(|entry| entry.expect("a directory entry is readable").path())
        .filter_map(|path| Some(path.file_stem()?.to_str()?.to_owned()))
        .collect();
    codes.sort();
    codes.extend(["L9999", "l0100", "L010", "x"].map(str::to_owned));
    codes
        .iter()
        .map(|code| Case::written(&format!("explain {code}")))
        .collect()
}

/// Each command that reads a path, on every directory under `tests/spec`.
fn directory_cases() -> Vec<Case> {
    let mut directories = Vec::new();
    gather_directories(Path::new("tests/spec"), &mut directories);
    directories.sort();
    ["check", "check --json", "api", "fmt", "build"]
        .iter()
        .flat_map(|command| {
            directories
                .iter()
                .map(move |directory| Case::written(&format!("{command} {directory}")))
        })
        .collect()
}

fn gather_directories(directory: &Path, found: &mut Vec<String>) {
    found.push(directory.display().to_string());
    for entry in fs::read_dir(repository().join(directory)).expect("a directory is readable") {
        let name = entry.expect("a directory entry is readable").file_name();
        let path = directory.join(name);
        if repository().join(&path).is_dir() {
            gather_directories(&path, found);
        }
    }
}

/// Each command that reads a path, on paths that name no file it can read.
fn unreadable_cases() -> Vec<Case> {
    let paths = [
        "no/such.lm",
        "compiler/",
        "compiler/main.lm/",
        "compiler/main.lm/x",
        "tests",
    ];
    [
        "check",
        "check --json",
        "api",
        "fmt",
        "build",
        "test",
        "run",
    ]
    .iter()
    .flat_map(|command| paths.map(|path| Case::written(&format!("{command} {path}"))))
    .collect()
}

/// Every file under `directory`, by its path from there, with its bytes.
fn files_under(directory: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut waiting = vec![directory.to_path_buf()];
    while let Some(at) = waiting.pop() {
        for entry in fs::read_dir(&at).expect("a directory of the stage is readable") {
            let path = entry.expect("a directory entry is readable").path();
            if path.is_dir() {
                waiting.push(path);
            } else {
                let bytes = fs::read(&path).expect("a file of the stage is readable");
                let relative = path
                    .strip_prefix(directory)
                    .expect("under the side")
                    .to_path_buf();
                found.insert(relative, bytes);
            }
        }
    }
    found
}

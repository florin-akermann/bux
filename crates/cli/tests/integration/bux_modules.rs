//! The loader written in Bux, `compiler/modules.lm`, held to the Rust loader's answer.
//!
//! `docs/specs/modules.md` and `docs/specs/packages.md` state the behaviour, and the Rust loader is
//! the answer the Bux one must give: the same modules in the same order, or the same refusal.
//! Building the Bux loader needs no JDK; running what was built needs one, and a test that runs
//! it is skipped with a named reason when `JAVA_HOME` names none.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use hegel::TestCase;
use hegel::generators as gs;
use lumen_modules::{NotLoaded, load};

use crate::common::{Example, Sibling, Within, as_argument, jdk, lumen, repository};

/// A program that writes, beside each file it is run with, what the Bux loader says of it.
///
/// One run loads every file, because each run is one compiler and one JVM.
const WRITES_THE_ANSWER: &str = "import files\n\nimport modules\n\nfn main(arguments: List<String>) -> Int {\n    for path in arguments {\n        _ = files.write(path + \".answer\", modules.printed(path))\n    }\n    0\n}\n";

/// The modules of the Bux compiler that the loader is, beside the program as siblings.
const MODULES: [&str; 4] = ["lexer", "ast", "parser", "modules"];

/// How many drawn trees the property loads, which is one compiler and one JVM each.
const CASES: u64 = 10;

/// A module that declares one function and imports nothing.
const DECLARING: &str = "fn hello() -> Int {\n    1\n}\n";

/// A module the grammar refuses.
const BROKEN: &str = "fn hello( -> Int {\n}\n";

/// One tree of files to load, and the file under it that loading starts from.
struct Case {
    named: &'static str,
    files: Vec<(String, String)>,
    root: &'static str,
}

/// A module that imports each of `names`, in the order it names them, and declares one function.
fn importing(names: &[&str]) -> String {
    let mut written = String::new();
    for name in names {
        let _ = write!(written, "import {name}\n\n");
    }
    written + DECLARING
}

/// The manifest of a package called `name`, stating `depends` as they are written.
fn manifest(name: &str, depends: &[&str]) -> String {
    let mut written = format!("package {name}\nversion 0.2.0\n");
    for depended in depends {
        let _ = writeln!(written, "depends {depended}");
    }
    written
}

/// One file of a case, at `at` under the case's own directory.
fn file(at: &str, content: &str) -> (String, String) {
    (at.to_owned(), content.to_owned())
}

/// Each case of the Rust loader's own tests, with the load order or the refusal it gives.
fn cases() -> Vec<Case> {
    let mut all = beside_cases();
    all.extend(file_cases());
    all.extend(library_cases());
    all.extend(package_cases());
    all.extend(claiming_cases());
    all.extend(manifest_cases());
    all
}

/// Modules beside each other: an order, a missing module, a ring, and a refused module.
fn beside_cases() -> Vec<Case> {
    vec![
        Case {
            named: "alone",
            files: vec![file("main.lm", DECLARING)],
            root: "main.lm",
        },
        Case {
            named: "chain",
            files: vec![
                file("main.lm", &importing(&["middle"])),
                file("middle.lm", &importing(&["bottom"])),
                file("bottom.lm", DECLARING),
            ],
            root: "main.lm",
        },
        Case {
            named: "diamond",
            files: vec![
                file("main.lm", &importing(&["left", "right"])),
                file("left.lm", &importing(&["shared"])),
                file("right.lm", &importing(&["shared"])),
                file("shared.lm", DECLARING),
            ],
            root: "main.lm",
        },
        Case {
            named: "missing",
            files: vec![
                file("main.lm", &importing(&["middle"])),
                file("middle.lm", &importing(&["bottom"])),
            ],
            root: "main.lm",
        },
        Case {
            named: "ring",
            files: vec![
                file("main.lm", &importing(&["middle"])),
                file("middle.lm", &importing(&["bottom"])),
                file("bottom.lm", &importing(&["main"])),
            ],
            root: "main.lm",
        },
        Case {
            named: "itself",
            files: vec![file("main.lm", &importing(&["main"]))],
            root: "main.lm",
        },
        Case {
            named: "broken",
            files: vec![
                file("main.lm", &importing(&["greeting"])),
                file("greeting.lm", BROKEN),
            ],
            root: "main.lm",
        },
    ]
}

/// Files and paths: a file not there, a directory named as a module is, and a `.` part.
fn file_cases() -> Vec<Case> {
    vec![
        Case {
            named: "unreadable",
            files: vec![file("main.lm", DECLARING)],
            root: "nothing.lm",
        },
        Case {
            named: "directory",
            files: vec![
                file("main.lm", &importing(&["held"])),
                file("held.lm/inside.lm", DECLARING),
            ],
            root: "main.lm",
        },
        Case {
            named: "dotted",
            files: vec![
                file("app/main.lm", &importing(&["beside"])),
                file("app/beside.lm", DECLARING),
            ],
            root: "app/./main.lm",
        },
    ]
}

/// Library modules, which the compiler carries, and the prelude, which nothing imports.
fn library_cases() -> Vec<Case> {
    vec![
        Case {
            named: "library",
            files: vec![file(
                "main.lm",
                &importing(&[
                    "environment",
                    "files",
                    "io",
                    "list",
                    "map",
                    "process",
                    "set",
                    "strings",
                ]),
            )],
            root: "main.lm",
        },
        Case {
            named: "prelude",
            files: vec![file("main.lm", &importing(&["prelude"]))],
            root: "main.lm",
        },
        Case {
            named: "shadow",
            files: vec![
                file("main.lm", &importing(&["list"])),
                file("list.lm", BROKEN),
            ],
            root: "main.lm",
        },
    ]
}

/// Packages: what a manifest reaches, what it does not, and two files claiming one name.
fn package_cases() -> Vec<Case> {
    vec![
        Case {
            named: "dependency",
            files: vec![
                file("app/main.lm", &importing(&["circle", "list"])),
                file("app/bux.package", &manifest("app", &["../shapes"])),
                file("shapes/circle.lm", &importing(&["point"])),
                file("shapes/bux.package", &manifest("shapes", &["../geometry"])),
                file("geometry/point.lm", DECLARING),
                file("geometry/bux.package", &manifest("geometry", &[])),
            ],
            root: "app/main.lm",
        },
        Case {
            named: "beside-first",
            files: vec![
                file("app/main.lm", &importing(&["circle"])),
                file("app/circle.lm", DECLARING),
                file("app/bux.package", &manifest("app", &["../shapes"])),
                file("shapes/circle.lm", BROKEN),
                file("shapes/bux.package", &manifest("shapes", &[])),
            ],
            root: "app/main.lm",
        },
        Case {
            named: "not-through",
            files: vec![
                file("app/main.lm", &importing(&["point"])),
                file("app/bux.package", &manifest("app", &["../shapes"])),
                file("shapes/circle.lm", DECLARING),
                file("shapes/bux.package", &manifest("shapes", &["../geometry"])),
                file("geometry/point.lm", DECLARING),
                file("geometry/bux.package", &manifest("geometry", &[])),
            ],
            root: "app/main.lm",
        },
    ]
}

/// Packages that reach one name twice: two files claiming it, one file along two routes.
fn claiming_cases() -> Vec<Case> {
    vec![
        Case {
            named: "two-dependencies",
            files: vec![
                file("app/main.lm", &importing(&["demo"])),
                file("app/bux.package", &manifest("app", &["../one", "../two"])),
                file("one/demo.lm", DECLARING),
                file("one/bux.package", &manifest("one", &[])),
                file("two/demo.lm", DECLARING),
                file("two/bux.package", &manifest("two", &[])),
            ],
            root: "app/main.lm",
        },
        Case {
            named: "two-routes",
            files: vec![
                file("app/main.lm", &importing(&["left", "right"])),
                file("app/bux.package", &manifest("app", &["../one", "../two"])),
                file("one/left.lm", &importing(&["shared"])),
                file("one/bux.package", &manifest("one", &["../common"])),
                file("two/right.lm", &importing(&["shared"])),
                file("two/bux.package", &manifest("two", &["./../common/."])),
                file("common/shared.lm", DECLARING),
                file("common/bux.package", &manifest("common", &[])),
            ],
            root: "app/main.lm",
        },
        Case {
            named: "claimed-twice",
            files: vec![
                file("app/main.lm", &importing(&["demo", "circle"])),
                file("app/demo.lm", DECLARING),
                file("app/bux.package", &manifest("app", &["../shapes"])),
                file("shapes/circle.lm", &importing(&["demo"])),
                file("shapes/demo.lm", DECLARING),
                file("shapes/bux.package", &manifest("shapes", &[])),
            ],
            root: "app/main.lm",
        },
        Case {
            named: "ring-across",
            files: vec![
                file("app/main.lm", &importing(&["circle"])),
                file("app/bux.package", &manifest("app", &["../shapes"])),
                file("shapes/circle.lm", &importing(&["circle"])),
                file("shapes/bux.package", &manifest("shapes", &[])),
            ],
            root: "app/main.lm",
        },
    ]
}

/// Manifests that are not written the way a manifest is, and one written with carriage returns.
fn manifest_cases() -> Vec<Case> {
    let stating = |named: &'static str, written: &str| Case {
        named,
        files: vec![
            file("app/main.lm", &importing(&["circle"])),
            file("app/bux.package", written),
            file("shapes/circle.lm", DECLARING),
            file("shapes/bux.package", &manifest("shapes", &[])),
        ],
        root: "app/main.lm",
    };
    vec![
        stating("empty", ""),
        stating("no-version", "package app\n"),
        stating("out-of-order", "version 1\npackage app\n"),
        stating(
            "two-words",
            "package app\nversion 1\ndepends ../shapes ../other\n",
        ),
        stating("no-word", "package app\nversion 1\ndepends \n"),
        stating(
            "blank-line",
            "package app\nversion 1\n\ndepends ../shapes\n",
        ),
        stating(
            "returns",
            "package app\r\nversion 1\r\ndepends ../shapes\r\n",
        ),
        stating("wide", "package é\r\nversion 1\u{a0}2\r\n"),
        stating(
            "twice",
            "package app\nversion 1\ndepends ../shapes\ndepends ../shapes/\n",
        ),
        stating("nowhere", "package app\nversion 1\ndepends ../gone\n"),
        stating("no-break", "package app\nversion 1\ndepends ../shapes"),
        stating("dot", "package app\nversion 1\ndepends ../shapes/.\n"),
        Case {
            named: "bad-dependency",
            files: vec![
                file("app/main.lm", &importing(&["circle"])),
                file("app/bux.package", &manifest("app", &["../shapes"])),
                file("shapes/circle.lm", DECLARING),
                file("shapes/bux.package", "package shapes\nrelease 1\n"),
            ],
            root: "app/main.lm",
        },
    ]
}

#[test]
fn the_bux_loader_builds_under_the_rust_compiler() {
    let program = the_bux_loader_beside(WRITES_THE_ANSWER);

    let run = lumen(&["build", as_argument(&program.path)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn every_example_the_bux_loader_states_holds() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the examples needs one");
        return;
    };

    let run = lumen(&[
        "test",
        as_argument(&repository().join("compiler/modules.lm")),
    ]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

/// `docs/specs/modules.md`: the Bux loader gives the Rust loader's order and refusal on each case.
#[test]
fn the_bux_loader_gives_the_rust_loaders_answer_on_every_case() {
    let program = the_bux_loader_beside(WRITES_THE_ANSWER);
    let roots: Vec<PathBuf> = cases()
        .iter()
        .map(|case| written_out(&program, case))
        .collect();

    let Some(answers) = answers_of_the_bux_loader(&program, &roots) else {
        return;
    };

    for (root, answer) in roots.iter().zip(answers) {
        assert_eq!(answer, rust_answer(root), "{}", root.display());
    }
}

/// `docs/specs/packages.md`: on a drawn tree of modules and manifests, the two loaders agree.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn the_bux_loader_gives_the_rust_loaders_answer_on_drawn_trees(tc: TestCase) {
    let program = the_bux_loader_beside(WRITES_THE_ANSWER);
    let roots: Vec<PathBuf> = (0..8)
        .map(|index| {
            let case = drawn_case(&tc);
            written_out_as(&program, &format!("drawn-{index}"), &case)
        })
        .collect();

    let Some(answers) = answers_of_the_bux_loader(&program, &roots) else {
        return;
    };

    for (root, answer) in roots.iter().zip(answers) {
        assert_eq!(answer, rust_answer(root), "{}", root.display());
    }
}

/// The names a drawn module is called, and imports.
const NAMES: [&str; 4] = ["a", "b", "c", "main"];

/// What a drawn module imports besides a drawn module: the library, the prelude, and nothing.
const OTHERS: [&str; 4] = ["list", "map", "prelude", "gone"];

/// The manifest a drawn package holds, where it holds one.
const MANIFESTS: [&str; 9] = [
    "package app\nversion 1\ndepends ../lib\n",
    "package app\nversion 1\n",
    "package app\r\nversion 1\r\ndepends ../lib\r\ndepends ../other\r\n",
    "package app\nversion 1\ndepends ../lib\ndepends ../lib/\n",
    "package app\nversion 1\ndepends ../none\n",
    "package  app\n",
    "version 1\n",
    "",
    "package app\nversion 1\ndepends ../other\n",
];

/// A tree of two or three packages, each module of which imports what was drawn for it.
fn drawn_case(tc: &TestCase) -> Case {
    let mut files = vec![file("app/main.lm", &drawn_module(tc))];
    for name in ["a", "b", "c"] {
        files.extend(drawn_modules_called(tc, name));
    }
    if tc.draw(gs::booleans()) {
        files.push(file(
            "app/bux.package",
            tc.draw(gs::sampled_from(&MANIFESTS)),
        ));
    }
    files.extend(drawn_dependencies(tc));
    Case {
        named: "drawn",
        files,
        root: "app/main.lm",
    }
}

/// A module called `name` in each package that was drawn to hold one.
fn drawn_modules_called(tc: &TestCase, name: &str) -> Vec<(String, String)> {
    ["app", "lib", "other"]
        .into_iter()
        .filter(|_| tc.draw(gs::booleans()))
        .map(|package| file(&format!("{package}/{name}.lm"), &drawn_module(tc)))
        .collect()
}

/// The manifest of each package a drawn manifest may depend on, where it was drawn to hold one.
fn drawn_dependencies(tc: &TestCase) -> Vec<(String, String)> {
    ["lib", "other"]
        .into_iter()
        .filter(|_| tc.draw(gs::booleans()))
        .map(|package| {
            file(
                &format!("{package}/bux.package"),
                &format!("package {package}\nversion 1\n"),
            )
        })
        .collect()
}

/// A module that imports up to three drawn names, or one the grammar refuses.
fn drawn_module(tc: &TestCase) -> String {
    let roll: u8 = tc.draw(gs::integers().min_value(0).max_value(9));
    if roll == 0 {
        return BROKEN.to_owned();
    }
    let every: Vec<&str> = NAMES.iter().chain(OTHERS.iter()).copied().collect();
    let imported: Vec<&str> = tc.draw(gs::vecs(gs::sampled_from(&every)).max_size(3));
    importing(&imported)
}

/// The files of `case`, written under a directory of their own, and the path loading starts at.
fn written_out(program: &Example, case: &Case) -> PathBuf {
    written_out_as(program, case.named, case)
}

/// The same, under the directory `named`.
fn written_out_as(program: &Example, named: &str, case: &Case) -> PathBuf {
    for (at, content) in &case.files {
        program.within_it(&Within {
            at: &format!("cases/{named}/{at}"),
            content,
        });
    }
    let directory = program.directory.join("cases").join(named);
    fs::create_dir_all(&directory).expect("a case's directory is creatable");
    directory.join(case.root)
}

/// What the Rust loader says of `root`, in the form the Bux loader prints its answer.
fn rust_answer(root: &Path) -> String {
    match load(root) {
        Ok(loaded) => {
            let mut written = "loaded\n".to_owned();
            for module in loaded.modules() {
                let _ = writeln!(written, "{} {}", module.name(), module.path().display());
            }
            written
        }
        Err(NotLoaded::Unreadable { path, .. }) => format!("unreadable {}\n", path.display()),
        Err(NotLoaded::Refused(refused)) => {
            let said = refused.diagnostic();
            let mut written = format!(
                "refused {}\n{} {}..{} {}\n",
                refused.path().display(),
                said.code().number(),
                said.span().start(),
                said.span().end(),
                said.message()
            );
            if let Some(help) = said.help() {
                let _ = writeln!(written, "help: {help}");
            }
            written
        }
    }
}

/// What the Bux loader says of each root, in order, and nothing where no JDK can run it.
///
/// The program is built with `lumen` and started on a JVM whose class path holds the repository
/// as well as the classes, which is what puts `library/` where the Bux loader asks for it.
fn answers_of_the_bux_loader(program: &Example, roots: &[PathBuf]) -> Option<Vec<String>> {
    let Some(java) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the Bux loader needs one");
        return None;
    };
    let built = lumen(&["build", as_argument(&program.path)]);
    assert_eq!(built.code, 0, "{}", built.stderr);
    let class_path = std::env::join_paths([program.directory.clone(), repository()])
        .expect("a class path joins");

    let ran = Command::new(java)
        .arg("--enable-preview")
        .arg("-cp")
        .arg(class_path)
        .arg("example")
        .args(roots)
        .output()
        .expect("the JVM starts");

    assert!(
        ran.status.success(),
        "{}",
        String::from_utf8_lossy(&ran.stderr)
    );
    Some(
        roots
            .iter()
            .map(|root| read(&PathBuf::from(format!("{}.answer", root.display()))))
            .collect(),
    )
}

/// A program of `content`, with each module of the Bux loader beside it.
fn the_bux_loader_beside(content: &str) -> Example {
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

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("an answer is UTF-8")
}

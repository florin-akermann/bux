//! The lowering and the class-file writer written in Bux, held to the bytes of the Rust phases.
//!
//! `compiler/ir.lm` lowers a program, `compiler/jvm.lm` writes its classes, and
//! `compiler/bytes.lm` writes the bytes. `docs/specs/codegen.md` states the behaviour, and the Rust
//! phases give the answer the Bux ones must give: the same class files, byte for byte, in the same
//! order. Building the Bux phases needs no JDK; running them needs one, and a test that runs them
//! is skipped with a named reason when `JAVA_HOME` names none.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use hegel::TestCase;
use hegel::generators as gs;
use lumen_holes::Whole;
use lumen_ir::{Asked, Lowered, lower, lower_prelude};
use lumen_jvm::ClassFile;
use lumen_modules::load;
use lumen_resolver::library::PRELUDE;
use lumen_types::{Imported, TypedProgram};

use crate::common::{Example, Sibling, Within, as_argument, jdk, lumen, repository};

/// The modules of the Bux compiler the writer reads, beside the program as siblings.
const MODULES: [&str; 18] = [
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
    "ir",
    "bytes",
    "jvm",
];

/// The modules whose examples this harness runs, which are the ones this phase adds.
const PHASE: [&str; 3] = ["ir", "bytes", "jvm"];

/// The directories whose every `.lm` file is a fixture, the Bux compiler's own modules among them.
const FIXTURES: [&str; 3] = ["tests/spec", "library", "compiler"];

/// How many runs a property makes, which is one compiler and one JVM each.
const CASES: u64 = 5;

/// How many drawn programs one run writes.
const PROGRAMS_A_RUN: usize = 8;

/// What a run says of a program the phases before the lowering refuse.
const SKIPPED: &str = "skipped\n";

/// What one program becomes: a line for each class file, then the bytes of all of them in order.
#[derive(Debug, PartialEq, Eq)]
struct Written {
    /// `path length` for each class file, or `skipped`.
    listing: String,
    bytes: Vec<u8>,
}

impl Written {
    /// The class files `classes` as a run writes them, or `skipped` where there are none.
    fn of(classes: Option<Vec<ClassFile>>) -> Self {
        let Some(classes) = classes else {
            return Self {
                listing: SKIPPED.to_owned(),
                bytes: Vec::new(),
            };
        };
        let mut listing = String::new();
        for class in &classes {
            let _ = writeln!(listing, "{} {}", class.path, class.bytes.len());
        }
        Self {
            listing,
            bytes: classes.into_iter().flat_map(|class| class.bytes).collect(),
        }
    }

    /// Where the bytes of the Bux run first differ from these, which is `None` where they agree.
    fn first_difference(&self, bux: &Self) -> Option<usize> {
        let differs = self.bytes.iter().zip(&bux.bytes).position(|(a, b)| a != b);
        differs.or_else(|| (self.bytes.len() != bux.bytes.len()).then_some(0))
    }
}

#[test]
fn the_bux_writer_builds_under_the_rust_compiler() {
    let program = the_bux_writer_beside(&writes_the_classes());

    let run = lumen(&["build", as_argument(&program.path)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn every_example_the_bux_writer_states_holds() {
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

/// `docs/specs/codegen.md`: on every fixture, the Bux writer writes the bytes the Rust one writes.
#[test]
fn the_bux_writer_writes_the_rust_bytes_on_every_fixture() {
    let program = the_bux_writer_beside(&writes_the_classes());
    let fixtures = fixtures();

    let Some(written) = written_by_the_bux_writer(&program, &fixtures) else {
        return;
    };

    for (fixture, bux) in fixtures.iter().zip(written) {
        the_same_classes(&Written::of(classes_by_rust(fixture)), &bux, fixture);
    }
}

/// `docs/specs/codegen.md`: on a drawn program, the two writers write the same bytes.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn the_bux_writer_writes_the_rust_bytes_on_drawn_programs(tc: TestCase) {
    let program = the_bux_writer_beside(&writes_the_classes());
    let roots: Vec<PathBuf> = (0..PROGRAMS_A_RUN)
        .map(|index| {
            let at = format!("drawn/{index}/main.lm");
            program.within_it(&Within {
                at: &at,
                content: &drawn_program(&tc),
            });
            program.directory.join(at)
        })
        .collect();

    let Some(written) = written_by_the_bux_writer(&program, &roots) else {
        return;
    };

    for (root, bux) in roots.iter().zip(written) {
        the_same_classes(&Written::of(classes_by_rust(root)), &bux, root);
    }
}

fn the_same_classes(rust: &Written, bux: &Written, fixture: &Path) {
    assert_eq!(bux.listing, rust.listing, "{}", fixture.display());
    assert_eq!(
        rust.first_difference(bux),
        None,
        "{}: the bytes first differ at this offset",
        fixture.display()
    );
}

/// The statements a drawn body writes before it gives back `total`.
const STATEMENTS: [&str; 9] = [
    "total += count",
    "total = total * 3 - count",
    "total = or(total / count, 7)",
    "if is_even(total) {\n        total += 1\n    }",
    "for index in [1, 2, 3] {\n        total += index\n    }",
    "total = match total {\n        0 => 1\n        other => other\n    }",
    "total = Box { held: total }.held",
    "total = total + strings.length(\"é€\" + shown(count))",
    "total = or(list.at([total, 2147483648, -40000], count), 128)",
];

/// A program whose `main` runs up to four drawn statements over a total.
fn drawn_program(tc: &TestCase) -> String {
    let statements: usize = tc.draw(gs::integers().min_value(0).max_value(4));
    let mut body = String::new();
    for _ in 0..statements {
        body.push_str("    ");
        body.push_str(tc.draw(gs::sampled_from(&STATEMENTS)));
        body.push('\n');
    }
    format!(
        "import list\n\nimport strings\n\nfn main(arguments: List<String>) -> Int {{\n    count := list.length(arguments)\n    var total = 0\n{body}    total\n}}\n\n/// Whether `total` is even.\n// example: is_even(2)\nfn is_even(total: Int) -> Bool {{\n    or(total % 2, 1) == 0\n}}\n\ntype Box = {{\n    held: Int\n}}\n"
    )
}

/// The class files the Rust phases write for the program at `path`, or `None` where a phase
/// before the lowering refuses it or a module holds a hole.
fn classes_by_rust(path: &Path) -> Option<Vec<ClassFile>> {
    let typed = typed_by_rust(path)?;
    let wholes = typed
        .iter()
        .map(|(module, program)| Some((module.as_str(), Whole::of_module(program).ok()?)))
        .collect::<Option<Vec<_>>>()?;
    Some(lumen_jvm::write(&lowered_by_rust(&wholes)))
}

/// Each module of the program at `path`, typed and checked, or `None` where a check refuses.
fn typed_by_rust(path: &Path) -> Option<Vec<(String, TypedProgram)>> {
    let mut imported = Imported::default();
    let mut typed = Vec::new();
    for module in load(path).ok()?.into_modules() {
        let resolved = lumen_resolver::resolve(module.program().clone(), module.name()).ok()?;
        let program = lumen_types::check(resolved, &imported).ok()?;
        lumen_exhaustiveness::check(&program).ok()?;
        imported = imported.offering(module.name(), program.surface().clone());
        typed.push((module.name().to_owned(), program));
    }
    Some(typed)
}

/// Every module and the prelude lowered, pass after pass, as a build lowers them.
fn lowered_by_rust(wholes: &[(&str, Whole<'_>)]) -> Vec<Lowered> {
    let mut asked = Asked::default();
    loop {
        let pass = Pass::over(wholes, asked);
        if pass.asks_nothing_new() {
            let mut lowered = pass.lowered;
            lowered.reverse();
            return lowered;
        }
        asked = pass.asked;
    }
}

/// One pass over every module and the prelude, as `crates/cli/src/lowering.rs` runs it.
struct Pass<'m> {
    lowered: Vec<Lowered>,
    asked: Asked,
    knew: Vec<(&'m str, Asked)>,
}

impl<'m> Pass<'m> {
    fn over(wholes: &[(&'m str, Whole<'_>)], asked: Asked) -> Self {
        let mut pass = Self {
            lowered: Vec::new(),
            asked,
            knew: Vec::new(),
        };
        for (module, whole) in wholes.iter().rev() {
            let one = lower(whole, module, &pass.asked);
            pass.add(module, one);
        }
        let prelude = lower_prelude(&pass.asked);
        pass.add(PRELUDE, prelude);
        pass
    }

    fn add(&mut self, module: &'m str, one: Lowered) {
        self.knew.push((module, self.asked.clone()));
        self.asked = std::mem::take(&mut self.asked).and(&one.asks);
        self.lowered.push(one);
    }

    fn asks_nothing_new(&self) -> bool {
        self.knew
            .iter()
            .all(|(module, knew)| !self.asked.asks_more_of(module, knew))
    }
}

/// A program that writes the class files of each path after the first, which names the directory
/// they go to: `<index>.classes` lists them, and `<index>.bytes` holds their bytes in order.
///
/// One run writes every program, because each run is one compiler and one JVM. The files are
/// written under the program's own directory, so nothing is written inside the repository.
fn writes_the_classes() -> String {
    let prelude = as_argument(&prelude_path())
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    format!(
        "import bytes\n\nimport files\n\nimport ir\n\nimport jvm\n\nimport list\n\nfn main(arguments: List<String>) -> Int {{\n    library := ir.library_read(ok_or(files.read(\"{prelude}\"), \"\"))\n    answers := or(list.at(arguments, 0), \".\")\n    var index = 0\n    for path in arguments {{\n        if index > 0 {{\n            var listing = \"\"\n            var all = []\n            for class in ok_or(jvm.classes_of(path, library), []) {{\n                listing = listing + class.path + \" \" + shown(list.length(class.bytes)) + \"\\n\"\n                all = bytes.all(all, class.bytes)\n            }}\n            if listing == \"\" {{\n                listing = \"{}\"\n            }}\n            _ = files.write(answers + \"/\" + shown(index) + \".classes\", listing)\n            _ = files.write_bytes(answers + \"/\" + shown(index) + \".bytes\", all)\n        }}\n        index += 1\n    }}\n    0\n}}\n",
        SKIPPED.replace('\n', "\\n")
    )
}

/// What the Bux writer writes for each root, in order, and nothing where no JDK can run it.
///
/// The program is built with `lumen` and started on a JVM whose class path holds the repository
/// as well as the classes, which is what puts `library/` where the Bux loader asks for it.
fn written_by_the_bux_writer(program: &Example, roots: &[PathBuf]) -> Option<Vec<Written>> {
    let Some(java) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the Bux writer needs one");
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
        .arg("-Xss512m")
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
            .map(|index| Written {
                listing: fs::read_to_string(answers.join(format!("{index}.classes")))
                    .expect("the program lists the classes of each root"),
                bytes: fs::read(answers.join(format!("{index}.bytes")))
                    .expect("the program writes the bytes of each root"),
            })
            .collect(),
    )
}

/// A program of `content`, with each module of the Bux writer beside it.
fn the_bux_writer_beside(content: &str) -> Example {
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

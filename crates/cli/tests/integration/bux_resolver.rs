//! The resolver written in Bux, `compiler/resolver.lm`, held to the Rust resolver's answer.
//!
//! `docs/specs/modules.md` states the behaviour and the printed form, and the Rust resolver is the
//! answer the Bux one must give: the same resolved names, or the same refusal. Building the Bux
//! resolver needs no JDK; running what was built needs one, and a test that runs it is skipped
//! with a named reason when `JAVA_HOME` names none.

use std::fs;
use std::path::{Path, PathBuf};

use hegel::TestCase;
use hegel::generators as gs;

use crate::common::{Example, Sibling, answers_of_one_run, as_argument, jdk, lumen, repository};

#[path = "../../../resolver/tests/integration/printed.rs"]
mod printed;

/// The modules of the Bux compiler that the resolver is, beside the program as siblings.
const MODULES: [&str; 4] = ["lexer", "ast", "parser", "resolver"];

/// The directories whose every `.lm` file that parses is a fixture.
const FIXTURES: [&str; 3] = ["tests/spec", "library", "compiler"];

/// How many runs the property makes, which is one compiler and one JVM each.
const CASES: u64 = 5;

/// How many drawn modules one run resolves.
const MODULES_A_RUN: usize = 24;

/// A module for each refusal of resolution, where the fixtures of `tests/spec` name none.
const REFUSALS: [&str; 17] = [
    "fn f() -> Money {\n    1\n}\n",
    "fn Ok() -> Int {\n    1\n}\n",
    "fn f(a: Int, a: Int) -> Int {\n    1\n}\n",
    "import io\n\nfn go() -> Int {\n    io.count\n}\n",
    "fn go() -> Int {\n    Marker = Marker\n    1\n}\n\ntype Marker = {\n}\n",
    "fn f(user: Int) -> user.User {\n    todo(\"x\")\n}\n",
    "fn f<T: Int<T>>(value: T) -> Int {\n    1\n}\n",
    "fn f(value: Eq) -> Int {\n    1\n}\n",
    "derive Eq for Int\n",
    "derive Add for Box\n\ntype Box = {\n    held: Int\n}\n",
    "instance Eq<Box<Int>> {\n    fn is_equal(one: Box<Int>, other: Box<Int>) -> Bool {\n        true\n    }\n}\n\ntype Box<T> = {\n    held: T\n}\n",
    "instance Eq<Box> {\n    fn is_equal(one: Box, other: Box) -> Bool {\n        true\n    }\n\n    fn is_equal(one: Box, other: Box) -> Bool {\n        true\n    }\n}\n\ntype Box = {\n}\n",
    "instance Eq<Box> {\n    fn equal(one: Box, other: Box) -> Bool {\n        true\n    }\n}\n\ntype Box = {\n}\n",
    "instance Eq<Box> {\n}\n\ntype Box = {\n}\n",
    "fn f(held: Int) -> Int {\n    match held {\n        1 | Some(other) => 1\n        _ => 2\n    }\n}\n",
    "fn f(held: Int) -> Int {\n    match held {\n        demo.Some(other) => 1\n        _ => 2\n    }\n}\n",
    "instance Show<Box> {\n    fn shown(value: Box) -> String {\n        \"box\"\n    }\n}\n\nderive Show for Box\n\ntype Box = {\n}\n",
];

#[test]
fn the_bux_resolver_builds_under_the_rust_compiler() {
    let program = the_bux_resolver_beside(&writes_the_answer());

    let run = lumen(&["build", as_argument(&program.path)]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn every_example_the_bux_resolver_states_holds() {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the examples needs one");
        return;
    };

    let run = lumen(&[
        "test",
        as_argument(&repository().join("compiler/resolver.lm")),
    ]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

/// `docs/specs/modules.md`: on every fixture, the Bux resolver gives the Rust resolver's answer.
#[test]
fn the_bux_resolver_gives_the_rust_resolvers_answer_on_every_fixture() {
    let mut sources: Vec<String> = fixtures()
        .iter()
        .map(|fixture| fs::read_to_string(fixture).expect("a fixture is UTF-8"))
        .filter(|source| lumen_parser::parse(source).is_ok())
        .collect();
    sources.extend(REFUSALS.iter().map(|&source| source.to_owned()));

    let Some(answers) = resolved_by_the_bux_resolver(&sources) else {
        return;
    };

    for (source, answer) in sources.iter().zip(answers) {
        assert_eq!(answer, rust_answer(source), "{source}");
    }
}

/// `docs/specs/modules.md`: on a drawn module, the two resolvers give one answer.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn the_bux_resolver_gives_the_rust_resolvers_answer_on_drawn_modules(tc: TestCase) {
    let sources: Vec<String> = (0..MODULES_A_RUN).map(|_| drawn_module(&tc)).collect();

    let Some(answers) = resolved_by_the_bux_resolver(&sources) else {
        return;
    };

    for (source, answer) in sources.iter().zip(answers) {
        assert_eq!(answer, rust_answer(source), "{source}");
    }
}

/// The names a drawn declaration is called, which clash, hide, and go missing often.
const DECLARED: [&str; 7] = ["f", "g", "h", "Id", "Box", "Pair", "Some"];

/// The names a drawn body writes, which are declared, supplied by the prelude, or nowhere.
const WRITTEN: [&str; 10] = [
    "held", "total", "f", "g", "Id", "Box", "io", "None", "or", "missing",
];

/// The types a drawn declaration writes.
const TYPES: [&str; 7] = ["Int", "Id", "Box", "Eq", "T", "io.Box", "Missing"];

/// The traits a drawn declaration names, from the prelude, from the module, and from nowhere.
const TRAITS: [&str; 5] = ["Eq", "Show", "Area", "Sized", "Add"];

/// The names a drawn trait is declared as, which the prelude does not hold.
const DECLARED_TRAITS: [&str; 2] = ["Area", "Sized"];

/// A module of up to five drawn declarations, which are written as the grammar writes them.
fn drawn_module(tc: &TestCase) -> String {
    let count: usize = tc.draw(gs::integers().min_value(1).max_value(5));
    let items: Vec<String> = (0..count).map(|_| drawn_item(tc)).collect();
    items.join("\n\n") + "\n"
}

/// One declaration of each kind the resolver walks.
fn drawn_item(tc: &TestCase) -> String {
    let name = pick(tc, &DECLARED);
    let type_name = pick(tc, &TYPES);
    let trait_name = pick(tc, &TRAITS);
    match tc.draw(gs::integers::<u8>().min_value(0).max_value(6)) {
        0 => format!("import {}", pick(tc, &["io", "list", "f"])),
        1 => format!("type {name} = {{\n    held: {type_name}\n}}"),
        2 => format!(
            "type {name}<T> =\n    | {}\n    | Held({type_name})",
            pick(tc, &DECLARED)
        ),
        3 => format!(
            "trait {}<T> {{\n    fn {name}(value: T) -> {type_name}\n}}",
            pick(tc, &DECLARED_TRAITS)
        ),
        4 => format!(
            "instance {trait_name}<{name}> {{\n    fn {}(value: {name}) -> Int {{\n        1\n    }}\n}}",
            pick(tc, &DECLARED)
        ),
        5 => format!("derive {trait_name} for {name}"),
        _ => drawn_function(tc, name, type_name),
    }
}

/// A function whose body binds, assigns, loops, calls, and matches drawn names.
fn drawn_function(tc: &TestCase, name: &str, type_name: &str) -> String {
    let statements: Vec<String> = tc
        .draw(gs::vecs(gs::integers::<u8>().min_value(0).max_value(5)).max_size(3))
        .into_iter()
        .map(|kind| format!("    {}\n", drawn_statement(tc, kind)))
        .collect();
    format!(
        "fn {name}<T: {}<T>>(held: {type_name}) -> Int {{\n{}    {}\n}}",
        pick(tc, &TRAITS),
        statements.concat(),
        drawn_expression(tc)
    )
}

/// One statement of a drawn body.
fn drawn_statement(tc: &TestCase, kind: u8) -> String {
    let bound = pick(tc, &WRITTEN);
    match kind {
        0 => format!("{bound} := {}", drawn_expression(tc)),
        1 => format!("var {bound} = {}", drawn_expression(tc)),
        2 => format!("{bound} = {}", drawn_expression(tc)),
        3 => format!(
            "for {bound} in {} {{\n        {}\n    }}",
            drawn_expression(tc),
            drawn_expression(tc)
        ),
        4 => format!(
            "_ = match {} {{\n        {} => 1\n        _ => 2\n    }}",
            pick(tc, &WRITTEN),
            drawn_pattern(tc)
        ),
        _ => format!("return {}", drawn_expression(tc)),
    }
}

/// The forms of a drawn expression: a name, a call, a dotted call, a dotted name, and a record.
const EXPRESSIONS: [&str; 5] = [
    "ONE",
    "ONE(OTHER)",
    "ONE.OTHER()",
    "ONE.OTHER",
    "ONE { held: OTHER }",
];

/// The forms of a drawn pattern: a binding, a constructor, an or-pattern, a record, a dotted name.
const PATTERNS: [&str; 5] = ["ONE", "Some(ONE)", "1 | ONE", "ONE { held }", "io.ONE"];

fn drawn_expression(tc: &TestCase) -> String {
    filled(tc, &EXPRESSIONS)
}

fn drawn_pattern(tc: &TestCase) -> String {
    filled(tc, &PATTERNS)
}

/// One of `forms`, with a drawn name for each `ONE` and `OTHER` in it.
fn filled(tc: &TestCase, forms: &[&str]) -> String {
    pick(tc, forms)
        .replace("ONE", pick(tc, &WRITTEN))
        .replace("OTHER", pick(tc, &WRITTEN))
}

fn pick<'w>(tc: &TestCase, among: &'w [&'w str]) -> &'w str {
    tc.draw(gs::sampled_from(among))
}

/// The Rust resolver's answer, in the form the Bux resolver prints its own.
///
/// The prelude's own source is resolved as the prelude, as the Bux program resolves it.
fn rust_answer(source: &str) -> String {
    if source == prelude_source() {
        return printed::render_prelude();
    }
    printed::render(source)
}

/// What the Bux resolver writes for each source, in order, and nothing where no JDK can run it.
fn resolved_by_the_bux_resolver(sources: &[String]) -> Option<Vec<String>> {
    if jdk().is_none() {
        eprintln!("skipped: JAVA_HOME names no JDK, and running the Bux resolver needs one");
        return None;
    }
    let program = the_bux_resolver_beside(&writes_the_answer());
    Some(answers_of_one_run(&program, sources))
}

/// A program that writes, beside each file it is run with, the answer of the Bux resolver.
///
/// One run resolves every file, because each run is one compiler and one JVM. The resolver reads
/// no file, so the program reads the prelude, at the path this harness writes into it.
fn writes_the_answer() -> String {
    let prelude = prelude_path();
    let path = as_argument(&prelude)
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    format!(
        "import ast\n\nimport files\n\nimport parser\n\nimport resolver\n\nfn main(arguments: List<String>) -> Int {{\n    library := ok_or(files.read(\"{path}\"), \"\")\n    prelude := resolver.prelude_of(ok_or(parser.parse(library), ast.Program {{ items: [] }}))\n    for path in arguments {{\n        source := ok_or(files.read(path), \"\")\n        var answer = resolver.printed(source, prelude)\n        if source == library {{\n            answer = resolver.prelude_printed(source)\n        }}\n        _ = files.write(path + \".answer\", answer)\n    }}\n    0\n}}\n"
    )
}

/// A program of `content`, with each module of the Bux resolver beside it.
fn the_bux_resolver_beside(content: &str) -> Example {
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

fn prelude_source() -> String {
    fs::read_to_string(prelude_path()).expect("the prelude is UTF-8")
}

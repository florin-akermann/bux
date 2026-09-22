//! The invariants of `docs/specs/grammar.md`, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ast::{Item, Program, Span};
use lumen_parser::parse;

fn source_text() -> impl hegel::Generator<String> {
    gs::text()
}

/// Every kind of member an `extern` reaches, each written with its name and result left open.
const KINDS: [&str; 4] = [
    "field {width}{name}() -> {result} = \"java.lang.System.out\"",
    "static {width}{name}(path: File) -> {result} = \"java.nio.file.Files.readString\"",
    "method {width}{name}(file: File) -> {result} = \"toPath\"",
    "new {width}{name}(path: String) -> {result}",
];

/// What a declaration says its member's own descriptor gives back, which is written or is not.
const WIDTHS: [&str; 2] = ["", "int "];

/// The names a generated declaration is written under, none of which the prelude declares.
///
/// `int` is among them because it is the width where a name follows it and a name where `(` does,
/// which is the one place the grammar of `docs/specs/grammar.md` reads a second token.
const DECLARED: [&str; 4] = ["reached", "held_by", "opened", "int"];

/// Every type a generated declaration gives back, which is what the boundary carries.
const RESULTS: [&str; 7] = [
    "Bool",
    "Int",
    "String",
    "File",
    "()",
    "Option<File>",
    "Result<File, String>",
];

/// One `extern` declaration, written under a name and giving back a type the boundary carries.
fn declaration(tc: &TestCase) -> String {
    let kind = tc.draw(gs::sampled_from(&KINDS));
    let name = tc.draw(gs::sampled_from(&DECLARED));
    let result = tc.draw(gs::sampled_from(&RESULTS));
    let written = kind
        .replace("{width}", tc.draw(gs::sampled_from(&WIDTHS)))
        .replace("{name}", name)
        .replace("{result}", result);
    format!("extern type File = \"java.io.File\"\n\nextern {written}\n")
}

#[hegel::test]
fn no_extern_declaration_writes_a_body(tc: TestCase) {
    let source = declaration(&tc);

    let parsed = parse(&source).expect("a generated declaration parses");

    assert!(
        parsed.functions().next().is_none(),
        "a declaration writes no function to hold a body"
    );
    let bodied = source.trim_end().to_owned() + " {\n}\n";
    assert!(
        parse(&bodied).is_err(),
        "a body written after a declaration is refused"
    );
}

#[hegel::test]
fn parsing_never_panics_and_is_deterministic(tc: TestCase) {
    let source = tc.draw(source_text());
    assert_eq!(parse(&source), parse(&source));
}

#[hegel::test]
fn an_error_span_is_non_empty_and_lies_within_the_source(tc: TestCase) {
    let source = tc.draw(source_text());
    let Err(error) = parse(&source) else {
        return;
    };
    let span = error.span();
    assert!(span.start() < span.end(), "{span:?} is empty");
    assert!(span.end() <= source.len(), "{span:?} runs past the input");
    assert!(source.is_char_boundary(span.start()) && source.is_char_boundary(span.end()));
}

/// The item shapes a generated program is built from, each one line of source.
const ITEMS: [&str; 9] = [
    "import io",
    "type UserId = UserId(Int)",
    "type User = {\n    id: UserId\n}",
    "type Payment =\n    | Pending\n    | Failed(String)",
    "fn identity(x) {\n    x\n}",
    "fn named(a: Int) -> Int {\n    joined(first: a, second: a)\n}",
    "fn total(users: List<User>) -> Int {\n    var total = 0\n    for user in users {\n        total += 1\n    }\n    total\n}",
    "fn describe(p: Payment) -> String {\n    match p {\n        Pending =>\n            \"waiting\"\n\n        Failed(reason) =>\n            reason\n    }\n}",
    "fn pick(a: Int) -> Int {\n    if a > 1 {\n        return a\n    } else {\n        return 0\n    }\n}",
];

/// A program of zero or more of [`ITEMS`], separated by a blank line.
fn well_formed_items(tc: &TestCase) -> Vec<&'static str> {
    tc.draw(gs::vecs(gs::sampled_from(&ITEMS)))
}

#[hegel::test]
fn a_well_formed_program_parses(tc: TestCase) {
    let items = well_formed_items(&tc);
    let source = items.join("\n\n");
    let program = parse(&source).unwrap_or_else(|error| {
        panic!("{source:?} parses: {}", error.message());
    });
    assert_eq!(program.items.len(), items.len());
}

#[hegel::test]
fn item_spans_are_ordered_non_overlapping_and_within_the_source(tc: TestCase) {
    let source = well_formed_items(&tc).join("\n\n");
    let program = parse(&source).expect("a well-formed program parses");
    let mut previous_end = 0;
    for span in item_spans(&program) {
        assert!(
            span.start() >= previous_end,
            "{span:?} overlaps the item before it"
        );
        assert!(span.start() < span.end(), "{span:?} is empty");
        assert!(span.end() <= source.len(), "{span:?} runs past the input");
        previous_end = span.end();
    }
}

fn item_spans(program: &Program) -> Vec<Span> {
    program
        .items
        .iter()
        .map(|item| match item {
            Item::Import(import) => import.span,
            Item::Type(declaration) => declaration.span,
            Item::Trait(declaration) => declaration.span,
            Item::Instance(declaration) => declaration.span,
            Item::Derive(declaration) => declaration.span,
            Item::Function(function) => function.span,
            Item::Extern(declaration) => declaration.span,
        })
        .collect()
}

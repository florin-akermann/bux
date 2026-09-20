//! The invariants of `docs/specs/types.md`, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_parser::parse;
use lumen_resolver::resolve;
use lumen_types::check;

use crate::common::{expressions, inferred_type, refusal};

/// The pieces a generated module is built from, each inferring on its own and declaring its own
/// names, so any set of them is a module that infers.
const PIECES: [&str; 6] = [
    "type UserId = UserId(Int)",
    "fn is_open(user: User) -> Bool {\n    user.active\n}\n\ntype User = {\n    id: Int\n    active: Bool\n}\n",
    "fn identity<T>(value: T) -> T {\n    value\n}",
    "fn total(counts: List<Int>) -> Int {\n    var sum = 0\n    for count in counts {\n        sum += count\n    }\n    sum\n}",
    "fn shout(word: String) -> String {\n    word + \"!\"\n}",
    "fn describe(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Failed(reason) => reason\n    }\n}\n\ntype Payment =\n    | Pending\n    | Failed(String)\n",
];

/// The pieces that are refused however sound the module around them is.
const REFUSALS: [&str; 4] = [
    "fn refused() -> Int {\n    \"seven\"\n}",
    "fn refused<T>(value: T) -> T {\n    1\n}",
    "fn is_refused(flag: Bool) -> Bool {\n    flag + flag\n}",
    "fn refused(value: Int) -> Int {\n    value.missing\n}",
];

#[hegel::test]
fn an_inference_never_panics_and_is_deterministic(tc: TestCase) {
    let source = tc.draw(gs::text());
    let Ok(program) = parse(&source) else {
        return;
    };
    let Ok(resolved) = resolve(program) else {
        return;
    };
    assert_eq!(check(resolved.clone()).is_ok(), check(resolved).is_ok());
}

#[hegel::test]
fn a_module_built_of_pieces_that_each_infer_infers(tc: TestCase) {
    let source = module(&tc);
    let resolved = resolved(&source);
    check(resolved).unwrap_or_else(|error| panic!("{source:?} infers: {}", error.message()));
}

#[hegel::test]
fn a_refusal_points_inside_the_source(tc: TestCase) {
    let refused = tc.draw(gs::sampled_from(&REFUSALS));
    let source = format!("{}\n\n{refused}\n", module(&tc));
    let error = check(resolved(&source))
        .err()
        .unwrap_or_else(|| panic!("{source:?} is refused"));
    let span = error.span();

    assert!(span.start() < span.end(), "{span:?} is empty");
    assert!(span.end() <= source.len(), "{span:?} runs past the input");
}

#[hegel::test]
fn every_expression_of_an_inferred_program_has_a_type(tc: TestCase) {
    let source = module(&tc);
    let typed = check(resolved(&source)).expect("a well-formed module infers");
    for expr in expressions(typed.resolved().program()) {
        assert!(
            typed.type_of(expr.span).is_some(),
            "{:?} has no type",
            expr.span.text(&source)
        );
    }
}

#[hegel::test]
fn no_expression_of_an_inferred_module_is_left_unsettled(tc: TestCase) {
    let source = module(&tc);
    let typed = check(resolved(&source)).expect("a well-formed module infers");
    for expr in expressions(typed.resolved().program()) {
        let inferred = typed.type_of(expr.span).expect("an expression has a type");
        assert!(
            !inferred.to_string().contains('_'),
            "{:?} was left unsettled as {inferred}",
            expr.span.text(&source)
        );
    }
}

/// A module of distinct [`PIECES`]; a piece drawn twice would declare its names twice.
fn module(tc: &TestCase) -> String {
    let mut chosen: Vec<&'static str> = Vec::new();
    for piece in tc.draw(gs::vecs(gs::sampled_from(&PIECES))) {
        if !chosen.contains(&piece) {
            chosen.push(piece);
        }
    }
    chosen.join("\n\n")
}

fn resolved(source: &str) -> lumen_resolver::ResolvedProgram {
    let program = parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {error:?}"));
    resolve(program).unwrap_or_else(|error| panic!("{source:?} resolves: {}", error.message()))
}

/// A value of each type a generic function can be used at, with the type it has.
const VALUES: [(&str, &str); 3] = [("1", "Int"), ("\"two\"", "String"), ("true", "Bool")];

/// A generic function is inferred to its most general type, so no pair of uses is too much for it.
#[hegel::test]
fn a_generic_function_is_general_enough_for_any_two_uses(tc: TestCase) {
    let (first, first_type) = tc.draw(gs::sampled_from(&VALUES));
    let (second, second_type) = tc.draw(gs::sampled_from(&VALUES));
    let source = format!(
        "fn is_general() -> {second_type} {{\n    one := given({first})\n    two := given({second})\n    two\n}}\n\n\
         fn given<T>(value: T) -> T {{\n    value\n}}\n"
    );

    assert_eq!(
        inferred_type(&source, &format!("given({first})"), 1),
        first_type
    );
    assert_eq!(
        inferred_type(&source, &format!("given({second})"), 1),
        second_type
    );
}

/// The statements a generated body is built from, each with the type it leaves behind.
///
/// `docs/specs/discarding.md` is about what a statement leaves, so the generator varies exactly
/// that: `()` from one, something worth reading from the next.
const STATEMENTS: [(&str, &str); 4] = [
    ("log(1)", "()"),
    ("()", "()"),
    ("save(1)", "Result<(), String>"),
    ("1 == 2", "Bool"),
];

#[hegel::test]
fn a_statement_that_leaves_a_value_is_refused_above_the_last_one(tc: TestCase) {
    let (above, left) = tc.draw(gs::sampled_from(&STATEMENTS));

    let refused = !accepts(&body("()", &format!("    {above}\n    ()")));

    assert_eq!(refused, left != UNIT, "{above} above `()` leaves {left}");
}

#[hegel::test]
fn the_last_statement_of_a_body_is_the_value_the_function_gives_back(tc: TestCase) {
    let (last, left) = tc.draw(gs::sampled_from(&STATEMENTS));

    assert!(
        accepts(&body(left, &format!("    {last}"))),
        "{last} leaves the {left} its function gives back"
    );
}

#[hegel::test]
fn an_underscore_and_an_equals_make_any_statement_compile(tc: TestCase) {
    let (thrown, left) = tc.draw(gs::sampled_from(&STATEMENTS));

    assert!(
        accepts(&body(UNIT, &format!("    _ = {thrown}\n    ()"))),
        "_ = {thrown} throws away {left}"
    );
}

/// The type of a statement that leaves nothing behind, which is what a discarded one must be.
const UNIT: &str = "()";

/// A module whose one function gives back `result` and holds `written`, over the ones it calls.
///
/// The function asks a question in its name because the generator varies its result type, and
/// `docs/specs/naming.md` holds a function that gives back a `Bool` to a name that asks one.
fn body(result: &str, written: &str) -> String {
    format!(
        concat!(
            "fn is_done() -> {result} {{\n{written}\n}}\n\n",
            "fn save(value: Int) -> Result<(), String> {{\n    Ok(())\n}}\n\n",
            "fn log(value: Int) -> () {{\n    ()\n}}\n"
        ),
        result = result,
        written = written
    )
}

/// Whether the whole front end has `source`, which is what `lumen check` answers.
fn accepts(source: &str) -> bool {
    let Ok(program) = parse(source) else {
        return false;
    };
    let Ok(resolved) = resolve(program) else {
        return false;
    };
    check(resolved).is_ok()
}

/// A value of each type a parameter may have, with the type it has.
///
/// `Bool` is not among them: `docs/specs/arguments.md` refuses a bare `Bool` parameter, so a
/// generated declaration that took one would be refused for a reason these properties are not
/// about.
const PASSABLE: [(&str, &str); 3] = [
    ("1", "Int"),
    ("\"two\"", "String"),
    ("Some(1)", "Option<Int>"),
];

/// A declaration taking `first` and `second`, beside a call of it passing `values` those types.
///
/// `docs/specs/arguments.md` is about the types a declaration repeats, so the generator varies
/// exactly that: two parameters of one type, or two parameters of two.
fn taking(first: &str, second: &str, call: &str) -> String {
    format!(
        "fn main() -> Int {{\n    {call}\n}}\n\n\
         fn takes(first: {first}, second: {second}) -> Int {{\n    1\n}}\n"
    )
}

#[hegel::test]
fn a_declaration_that_repeats_a_type_is_called_by_name_and_not_in_order(tc: TestCase) {
    let (value, of_type) = tc.draw(gs::sampled_from(&PASSABLE));

    let named = format!("takes(first: {value}, second: {value})");
    let in_order = format!("takes({value}, {value})");

    assert!(accepts(&taking(of_type, of_type, &named)), "{named}");
    assert!(!accepts(&taking(of_type, of_type, &in_order)), "{in_order}");
}

#[hegel::test]
fn a_declaration_whose_parameter_types_differ_is_called_either_way(tc: TestCase) {
    let (first, first_type) = tc.draw(gs::sampled_from(&PASSABLE));
    let (second, second_type) = tc.draw(gs::sampled_from(&PASSABLE));
    if first_type == second_type {
        return;
    }

    let named = format!("takes(first: {first}, second: {second})");
    let in_order = format!("takes({first}, {second})");

    assert!(accepts(&taking(first_type, second_type, &named)), "{named}");
    assert!(
        accepts(&taking(first_type, second_type, &in_order)),
        "{in_order}"
    );
}

#[hegel::test]
fn a_call_that_compiles_still_compiles_with_its_arguments_named(tc: TestCase) {
    let (first, first_type) = tc.draw(gs::sampled_from(&PASSABLE));
    let (second, second_type) = tc.draw(gs::sampled_from(&PASSABLE));
    let in_order = format!("takes({first}, {second})");
    if !accepts(&taking(first_type, second_type, &in_order)) {
        return;
    }

    let named = format!("takes(first: {first}, second: {second})");

    assert!(accepts(&taking(first_type, second_type, &named)), "{named}");
}

/// A signature built out of `Bool` and one other type, which is what the flag rule reads.
const EITHER: [&str; 2] = ["Bool", "Int"];

/// What a declaration takes and what it gives back, which is all the flag rule reads.
struct Signature<'a> {
    takes: Vec<&'a str>,
    gives: &'a str,
}

impl Signature<'_> {
    /// The declaration written out, with a body of `todo` so its signature is all that varies.
    fn declaration(&self) -> String {
        let written: Vec<String> = self
            .takes
            .iter()
            .enumerate()
            .map(|(position, one)| format!("p{position}: {one}"))
            .collect();
        format!(
            "fn is_open({}) -> {} {{\n    todo(\"the body is not the point\")\n}}\n",
            written.join(", "),
            self.gives
        )
    }

    /// Whether `docs/specs/arguments.md` refuses it, read as the spec states it.
    ///
    /// A parameter that is a `Bool` is a flag unless every type the signature mentions is `Bool`,
    /// which is the boolean operation the spec carves out.
    fn takes_a_flag(&self) -> bool {
        let mentioned = self.takes.iter().chain([&self.gives]);
        self.takes.contains(&"Bool") && !mentioned.into_iter().all(|one| *one == "Bool")
    }
}

/// What `L0412` tells an author to write instead, which is how a refusal is told from any other.
const FLAG_HELP: &str =
    "declare a two-variant type and take that instead, so the call says which of the two";

#[hegel::test]
fn a_bool_parameter_compiles_only_where_the_whole_signature_is_bool(tc: TestCase) {
    let signature = Signature {
        takes: tc.draw(gs::vecs(gs::sampled_from(&EITHER))),
        gives: tc.draw(gs::sampled_from(&EITHER)),
    };
    let source = signature.declaration();

    if signature.takes_a_flag() {
        assert_eq!(refusal(&source).help(), FLAG_HELP, "{source}");
    } else {
        assert!(accepts(&source), "{source}");
    }
}

/// The openers a generated name is built with, four of which are the questions a name may ask.
const OPENERS: [&str; 6] = ["is_", "has_", "can_", "should_", "was_", ""];

/// The words a generated name is built from, none of them asking a question on its own.
const WORDS: [&str; 3] = ["active", "paid", "island"];

/// The results a generated function gives back, one of which holds it to a question.
const GIVES: [&str; 2] = ["Bool", "Int"];

#[hegel::test]
fn a_bool_function_compiles_exactly_when_its_name_asks_a_question(tc: TestCase) {
    let opener = tc.draw(gs::sampled_from(&OPENERS));
    let word = tc.draw(gs::sampled_from(&WORDS));
    let gives = tc.draw(gs::sampled_from(&GIVES));
    let source = format!(
        "fn {opener}{word}(count: Int) -> {gives} {{\n    {}\n}}\n",
        if gives == "Bool" {
            "count > 1"
        } else {
            "count"
        }
    );

    let asks = ["is_", "has_", "can_", "should_"].contains(&opener);

    assert_eq!(accepts(&source), asks || gives != "Bool", "{source}");
}

/// Every name the two supplied modules declare, with the module each is reached through.
const SUPPLIED: [(&str, &str, &str); 3] = [
    ("io", "print", "()"),
    ("io", "println", "()"),
    ("files", "read", "Result<String, String>"),
];

/// The letters a generated name is drawn from, which are the ones a Lumen name may hold.
const NAME_LETTERS: &str = "adenoprstuw";

/// How long a generated name runs past the letter it opens with.
const NAME_LONGEST: usize = 6;

/// Where a call is written, which is every place an expression of its type may go.
const PLACES: [&str; 3] = [
    "    held := {call}\n    _ = held\n",
    "    _ = {call}\n",
    "    if true {\n        _ = {call}\n    }\n",
];

#[hegel::test]
fn a_name_a_supplied_module_declares_has_one_type_wherever_it_is_written(tc: TestCase) {
    let (module, name, result) = tc.draw(gs::sampled_from(&SUPPLIED));
    let place = tc.draw(gs::sampled_from(&PLACES));
    let call = format!("{module}.{name}(\"text\")");
    let body = place.replace("{call}", &call);
    let source = format!("import {module}\n\nfn go() -> () {{\n{body}}}\n");

    crate::common::inferred(&source);
    assert_eq!(
        inferred_type(&source, &call, 1),
        *result,
        "{module}.{name} gives back what `docs/specs/io.md` says it does"
    );
}

#[hegel::test]
fn a_name_a_supplied_module_does_not_declare_is_refused_naming_the_module_and_it(tc: TestCase) {
    let (module, _, _) = tc.draw(gs::sampled_from(&SUPPLIED));
    let opener = tc.draw(gs::sampled_from(&["a", "w", "r"]));
    let rest: String = tc.draw(gs::text().alphabet(NAME_LETTERS).max_size(NAME_LONGEST));
    let name = format!("{opener}{rest}");
    tc.assume(!SUPPLIED.iter().any(|(_, declared, _)| *declared == name));
    let source = format!("import {module}\n\nfn go() -> () {{\n    _ = {module}.{name}()\n}}\n");

    assert_eq!(
        refusal(&source).message(),
        format!("`{module}` declares no `{name}`")
    );
}

/// The names a generated ring or chain of records is declared under, in the order it runs.
const NAMED: [&str; 5] = ["Room", "Wing", "Floor", "Plan", "Site"];

/// A module declaring `many` records, each holding the next, and the last holding `last`.
fn along(many: usize, last: &str) -> String {
    let declared = NAMED.iter().take(many).enumerate().map(|(at, named)| {
        let held = if at + 1 == many { last } else { NAMED[at + 1] };
        format!("\ntype {named} = {{\n    count: Int\n    next: {held}\n}}\n")
    });
    let reached = NAMED[0];
    let opening = format!("fn counted(held: {reached}) -> Int {{\n    held.count\n}}\n");
    declared.fold(opening, |mut source, written| {
        source.push_str(&written);
        source
    })
}

#[hegel::test]
fn a_ring_of_records_is_refused_however_long_the_ring_is(tc: TestCase) {
    let many = tc.draw(gs::sampled_from(&[1, 2, 3, 4, 5]));

    let message = refusal(&along(many, NAMED[0])).message();

    let first = NAMED[0];
    assert!(
        message.starts_with(&format!("`{first}` holds ")),
        "{message}"
    );
    assert!(message.ends_with(&format!("`{first}`")), "{message}");
    assert_eq!(
        message.matches(", which holds ").count(),
        many - 1,
        "a ring of {many} names every one of them: {message}"
    );
}

#[hegel::test]
fn a_chain_of_records_that_never_comes_back_round_is_accepted(tc: TestCase) {
    let many = tc.draw(gs::sampled_from(&[1, 2, 3, 4, 5]));

    let source = along(many, "Int");

    assert!(check(resolve(parse(&source).expect("it parses")).expect("it resolves")).is_ok());
}

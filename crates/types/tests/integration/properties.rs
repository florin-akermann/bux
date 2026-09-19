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
    "fn open(user: User) -> Bool {\n    user.active\n}\n\ntype User = {\n    id: Int\n    active: Bool\n}\n",
    "fn identity<T>(value: T) -> T {\n    value\n}",
    "fn total(counts: List<Int>) -> Int {\n    var sum = 0\n    for count in counts {\n        sum += count\n    }\n    sum\n}",
    "fn shout(word: String) -> String {\n    word + \"!\"\n}",
    "fn describe(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Failed(reason) => reason\n    }\n}\n\ntype Payment =\n    | Pending\n    | Failed(String)\n",
];

/// The pieces that are refused however sound the module around them is.
const REFUSALS: [&str; 4] = [
    "fn refused() -> Int {\n    \"seven\"\n}",
    "fn refused<T>(value: T) -> T {\n    1\n}",
    "fn refused(flag: Bool) -> Bool {\n    flag + flag\n}",
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
        "fn go() -> {second_type} {{\n    one := given({first})\n    two := given({second})\n    two\n}}\n\n\
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

/// A module whose `main` gives back `result` and holds `written`, over the functions it calls.
fn body(result: &str, written: &str) -> String {
    format!(
        concat!(
            "fn main() -> {result} {{\n{written}\n}}\n\n",
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
            "fn open({}) -> {} {{\n    todo(\"the body is not the point\")\n}}\n",
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

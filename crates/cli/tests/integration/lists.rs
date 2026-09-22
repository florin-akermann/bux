//! `docs/specs/library.md`: a list grows by `list.push` and is read at an index by `list.at`.
//!
//! Both are the compiler's, written out where they are called, so what each does is observable
//! only by running a program that calls it. Each case draws the values a list is grown from,
//! writes the one program that grows it and reads it back, runs that program, and compares what
//! it wrote with what the drawn values say it had to write.

use hegel::TestCase;
use hegel::generators as gs;

use crate::common::{Example, jdk, lumen};

/// How many programs a property compiles and runs, which is one compiler and one JVM each.
///
/// A case here costs what a whole build costs rather than what a function call costs, so these
/// count in tens where a property over plain values counts in hundreds.
const CASES: u64 = 40;

/// What a program writes for an index the list holds nothing at, which no value it holds is.
const MISSING: i64 = -1000;

/// The value one push puts on the end, which no drawn value is.
const ONE: i64 = 101;

/// The value the other push puts on the end, which is neither a drawn value nor the one above.
const OTHER: i64 = 202;

/// `docs/specs/library.md` properties 5 and 6: what was pushed on reads back where it went.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn a_list_grown_by_push_reads_back_at_every_index_it_holds(tc: TestCase) {
    let grown = drawn(&tc);

    let Some(written) = ran(&a_list_grown_from(&grown)) else {
        return;
    };

    assert_eq!(written, read_out_of(&grown));
}

/// `docs/specs/library.md` property 6: a list is a value, so a push reaches into none.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn a_push_leaves_the_list_it_was_handed_holding_what_it_held(tc: TestCase) {
    let grown = drawn(&tc);

    let Some(written) = ran(&two_pushes_onto(&grown)) else {
        return;
    };

    assert_eq!(written, read_out_of_both(&grown));
}

/// The values a case grows a list out of, small enough to read in a counterexample.
fn drawn(tc: &TestCase) -> Vec<i64> {
    tc.draw(
        gs::vecs(gs::integers::<i64>().min_value(-99).max_value(99))
            .min_size(0)
            .max_size(6),
    )
}

/// Every index a case reads the list at: the ones it holds, and one past each end of them.
fn indices(held: usize) -> Vec<i64> {
    let last = i64::try_from(held).expect("a case grows a list of at most six values");
    (-2..=last + 1).collect()
}

/// A program that grows a list out of `grown` and writes what it reads at every index.
fn a_list_grown_from(grown: &[i64]) -> String {
    let read: Vec<String> = indices(grown.len())
        .iter()
        .map(|index| read_at("held", *index))
        .collect();
    a_program(&pushed_onto("[]", grown), &read)
}

/// What the list grown out of `grown` has to write: its values, and nothing past either end.
fn read_out_of(grown: &[i64]) -> String {
    let written: Vec<String> = indices(grown.len())
        .iter()
        .map(|index| held_at(grown, *index).to_string())
        .collect();
    lines(&written)
}

/// A program that pushes two values onto one list and reads all three of the lists back.
fn two_pushes_onto(grown: &[i64]) -> String {
    let last = i64::try_from(grown.len()).expect("a case grows a list of at most six values");
    let read = vec![
        "list.length(held)".to_owned(),
        read_at("held", last),
        read_at(&format!("list.push(held, {ONE})"), last),
        read_at(&format!("list.push(held, {OTHER})"), last),
        read_at("held", last),
    ];
    a_program(&pushed_onto("[]", grown), &read)
}

/// What that program has to write: the list is as long as it was, and holds what it held.
fn read_out_of_both(grown: &[i64]) -> String {
    let last = i64::try_from(grown.len()).expect("a case grows a list of at most six values");
    lines(&[
        last.to_string(),
        MISSING.to_string(),
        ONE.to_string(),
        OTHER.to_string(),
        MISSING.to_string(),
    ])
}

/// `values` pushed onto `built` one at a time, in the order they were drawn.
fn pushed_onto(built: &str, values: &[i64]) -> String {
    let mut grown = built.to_owned();
    for value in values {
        grown = format!("list.push({grown}, {value})");
    }
    grown
}

/// What `values` reads at `index`, and `MISSING` where it holds nothing there.
fn read_at(values: &str, index: i64) -> String {
    format!("or(list.at({values}, {index}), {MISSING})")
}

/// What `grown` holds at `index`, and `MISSING` where it holds nothing there.
fn held_at(grown: &[i64], index: i64) -> i64 {
    usize::try_from(index)
        .ok()
        .and_then(|at| grown.get(at))
        .copied()
        .unwrap_or(MISSING)
}

/// The program that binds `built` and writes each of `read`, one to a line.
fn a_program(built: &str, read: &[String]) -> String {
    let written: Vec<String> = read
        .iter()
        .map(|one| format!("    io.println(shown({one}))\n"))
        .collect();
    let body = written.concat();
    format!(
        "import io\n\nimport list\n\nfn main(arguments: List<String>) -> Int {{\n    held := {built}\n{body}    0\n}}\n"
    )
}

/// The words, one to a line, in the order the program writes them.
fn lines(written: &[String]) -> String {
    let each: Vec<String> = written.iter().map(|one| format!("{one}\n")).collect();
    each.concat()
}

/// What `source` writes when it is run, or nothing at all where there is no JDK to run it.
fn ran(source: &str) -> Option<String> {
    if jdk().is_none() {
        eprintln!("skipped: JAVA_HOME names no JDK, and running what was written needs one");
        return None;
    }
    let example = Example::new(source);
    let run = lumen(&["run", example.path.to_str().expect("a UTF-8 path")]);
    assert_eq!(run.code, 0, "{source} does not run:\n{}", run.stderr);
    Some(run.stdout)
}

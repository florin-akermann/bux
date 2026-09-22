//! `docs/specs/collections.md`: a map and a set give back exactly what was put into them.
//!
//! The library is Bux source, so what it does is observable only by running a program that
//! uses it. Each case draws what goes in, writes the one program that puts all of it in and
//! reads it all back out, runs that program, and compares what it wrote with what the drawn
//! entries say it had to write.

use std::collections::HashMap;

use hegel::TestCase;
use hegel::generators as gs;

use crate::common::{Example, jdk, lumen};

/// How many programs a property compiles and runs, which is one compiler and one JVM each.
///
/// A case here costs what a whole build costs rather than what a function call costs, so these
/// count in tens where a property over plain values counts in hundreds.
/// Shrinking is left out for the same reason: it runs hundreds more of them to make a draw of at
/// most six entries smaller, and a draw that small is already a counterexample a reader reads.
const CASES: u64 = 40;

/// `docs/specs/collections.md` properties 1 and 2: a key reads back as what last went in at it.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn a_map_gives_back_the_value_last_put_in_at_each_key(tc: TestCase) {
    let put_in = drawn(&tc);

    let Some(written) = ran(&a_map_of(&put_in)) else {
        return;
    };

    assert_eq!(written, read_out_of_a_map(&put_in));
}

/// `docs/specs/collections.md` property 3: a set holds what went into it and nothing else.
#[hegel::test(test_cases = CASES, phases = [hegel::Phase::Generate])]
fn a_set_holds_every_value_put_into_it_and_no_other(tc: TestCase) {
    let put_in = tc.draw(gs::vecs(values()).max_size(6));
    let asked = asked_of(&put_in, tc.draw(gs::vecs(values()).max_size(4)));

    let Some(written) = ran(&a_set_of(&put_in, &asked)) else {
        return;
    };

    assert_eq!(written, read_out_of_a_set(&put_in, &asked));
}

/// What a case puts in: a key to put a value at, and the whole number to put there.
///
/// The keys are drawn from a handful, so a case puts two values at one key often enough that
/// replacing one is what most cases read rather than a case of its own.
fn drawn(tc: &TestCase) -> Vec<(i64, i64)> {
    tc.draw(gs::vecs(gs::tuples2(gs::sampled_from(keys()), values())).max_size(6))
}

/// Every key a case may put a value at, which is every key it reads back out.
fn keys() -> Vec<i64> {
    (0..5).collect()
}

/// The whole numbers a case puts in, small enough to read in a counterexample.
fn values() -> impl gs::Generator<i64> {
    gs::integers::<i64>().min_value(-99).max_value(99)
}

/// What a case asks the set about: every value put in, and the ones drawn beside them.
///
/// The values put in are asked about so that a set that lost one is caught, and the ones drawn
/// beside them so that a set answering `true` to everything is caught as well.
fn asked_of(put_in: &[i64], beside: Vec<i64>) -> Vec<i64> {
    let mut asked = put_in.to_vec();
    asked.extend(beside);
    asked
}

/// What a program writes for a key the map holds no entry for, which no value it holds is.
const MISSING: i64 = -1000;

/// A program that puts `put_in` into a map in order and writes what every key reads back as.
fn a_map_of(put_in: &[(i64, i64)]) -> String {
    let mut built = "map.empty()".to_owned();
    for (key, value) in put_in {
        built = format!("map.insert({built}, {key}, {value})");
    }
    a_program("map", &built, &keys(), &|key| {
        format!("or(map.get(held, {key}), {MISSING})")
    })
}

/// What the map built out of `put_in` has to write: the last value put in at each key.
fn read_out_of_a_map(put_in: &[(i64, i64)]) -> String {
    let mut last: HashMap<i64, i64> = HashMap::new();
    for (key, value) in put_in {
        last.insert(*key, *value);
    }
    written(&keys(), &|key| {
        last.get(&key).copied().unwrap_or(MISSING).to_string()
    })
}

/// A program that puts `put_in` into a set in order and writes what it says of each of `asked`.
fn a_set_of(put_in: &[i64], asked: &[i64]) -> String {
    let mut built = "set.empty()".to_owned();
    for value in put_in {
        built = format!("set.insert({built}, {value})");
    }
    a_program("set", &built, asked, &|value| {
        format!("set.has_value(held, {value})")
    })
}

/// What the set built out of `put_in` has to write: whether each of `asked` went into it.
fn read_out_of_a_set(put_in: &[i64], asked: &[i64]) -> String {
    written(asked, &|value| put_in.contains(&value).to_string())
}

/// The program that binds `built` and writes `read` of each of `asked`, one to a line.
fn a_program(module: &str, built: &str, asked: &[i64], read: &dyn Fn(i64) -> String) -> String {
    let lines: Vec<String> = asked
        .iter()
        .map(|one| format!("    io.println(shown({}))\n", read(*one)))
        .collect();
    let reading = lines.concat();
    format!(
        "import io\n\nimport {module}\n\nfn main(arguments: List<String>) -> Int {{\n    held := {built}\n{reading}    0\n}}\n"
    )
}

/// The reading of each of `asked`, one to a line, in the order the program writes them.
fn written(asked: &[i64], read: &dyn Fn(i64) -> String) -> String {
    let lines: Vec<String> = asked
        .iter()
        .map(|one| format!("{}\n", read(*one)))
        .collect();
    lines.concat()
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

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

/// How many programs the property over hundreds of keys compiles and runs.
///
/// One of these puts hundreds of keys into one map and one set rather than a handful, so it
/// costs more than one of the cases above and fewer of them are run.
const WIDE_CASES: u64 = 8;

/// How many whole numbers a wide case draws, which is one pair of keys for each of them.
const DRAWN_KEYS: usize = 150;

/// `docs/specs/collections.md` property 4: hundreds of keys, some hashing alike, all read back.
///
/// One program answers for all three of the properties above at once: what `get` gives at each
/// key that went in, what it gives where a key went in twice, and what the set says of a value
/// put in once, twice, or not at all.
#[hegel::test(test_cases = WIDE_CASES, phases = [hegel::Phase::Generate])]
fn hundreds_of_keys_read_back_out_of_a_map_and_a_set_however_many_hash_alike(tc: TestCase) {
    let put_in = many_keys(&tc);
    let asked = asked_about(&put_in);

    let Some(written) = ran(&a_map_and_a_set_keyed_by(&put_in, &asked)) else {
        return;
    };

    assert_eq!(written, read_out_by_key(&put_in, &asked));
}

/// One key of a wide case: the two whole numbers the list holding it is written with.
type Key = (i64, i64);

/// The keys a wide case puts in, two of which hash alike for every whole number drawn.
///
/// The prelude works a list's hash out as `code * 31 + hashed(held)` from `1`, so `[one, 31]`
/// and `[one + 1, 0]` are one number and the two of them share a leaf of the trie. A number
/// drawn twice puts one key in twice, which is what reads the replacing of a value back.
fn many_keys(tc: &TestCase) -> Vec<Key> {
    let drawn: Vec<i64> = tc.draw(
        gs::vecs(gs::integers::<i64>().min_value(0).max_value(999))
            .min_size(DRAWN_KEYS)
            .max_size(DRAWN_KEYS),
    );
    let mut keys = Vec::new();
    for one in drawn {
        keys.push((one, 31));
        keys.push((one + 1, 0));
    }
    keys
}

/// The keys a wide case asks about: every key put in, and one beside it that was not.
///
/// No key put in is written with `30`, so each of those is a key the map holds no entry for and
/// the set holds no value equal to.
fn asked_about(put_in: &[Key]) -> Vec<Key> {
    let mut asked = put_in.to_vec();
    for (one, _) in put_in {
        asked.push((*one, 30));
    }
    asked
}

/// A program that puts `put_in` into a map and a set, then reads both at each of `asked`.
fn a_map_and_a_set_keyed_by(put_in: &[Key], asked: &[Key]) -> String {
    let keys = written_keys(put_in);
    let asked = written_keys(asked);
    format!(
        "import io\n\nimport map\n\nimport set\n\nfn main(arguments: List<String>) -> Int {{\n    keys := [{keys}]\n    asked := [{asked}]\n    var held = map.empty()\n    var seen = set.empty()\n    var at = 0\n    for key in keys {{\n        held = map.insert(held, key, at)\n        seen = set.insert(seen, key)\n        at = at + 1\n    }}\n    for key in asked {{\n        io.println(shown(or(map.get(held, key), {MISSING})))\n    }}\n    for key in asked {{\n        io.println(shown(set.has_value(seen, key)))\n    }}\n    0\n}}\n"
    )
}

/// `keys` as the list literals a program writes them with, in the order they are written.
fn written_keys(keys: &[Key]) -> String {
    let each: Vec<String> = keys
        .iter()
        .map(|(one, other)| format!("[{one}, {other}]"))
        .collect();
    each.join(", ")
}

/// What the map and the set built out of `put_in` have to write for each of `asked`.
fn read_out_by_key(put_in: &[Key], asked: &[Key]) -> String {
    let mut last: HashMap<Key, usize> = HashMap::new();
    for (at, key) in put_in.iter().enumerate() {
        last.insert(*key, at);
    }
    let values = each_line(asked, &|key| {
        last.get(key)
            .map_or_else(|| MISSING.to_string(), usize::to_string)
    });
    let held = each_line(asked, &|key| last.contains_key(key).to_string());
    values + &held
}

/// The reading of each of `asked`, one to a line, in the order the program writes them.
fn each_line(asked: &[Key], read: &dyn Fn(&Key) -> String) -> String {
    let lines: Vec<String> = asked.iter().map(|key| format!("{}\n", read(key))).collect();
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

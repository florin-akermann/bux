//! The bootstrap: the Bux compiler that `lumen` builds builds itself, and the two agree.
//!
//! `docs/specs/run.md` states the behaviour. Stage 1 is the compiler `lumen build` writes, and
//! stage 2 is the compiler that stage 1 writes, each in a stage of its own outside the repository.
//! Stage 2 is stage 1 byte for byte, and both run every program under `tests/spec` as `lumen`
//! does: this file holds stage 2 to that, and `bux_launcher.rs` holds stage 1. Stage 1 runs on a
//! JVM, so each test is skipped with a named reason when `JAVA_HOME` names no JDK.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::common::{
    LAUNCHED, Stage, files_ending_in, first_difference, fixture_cases, jdk,
    the_same_from_the_launcher,
};

/// Where a stage holds its classes, beside the sources they are built from.
const CLASSES: &str = "compiler";

/// `docs/specs/run.md`: stage 2 has the class files of stage 1, at the same paths, byte for byte.
#[test]
fn stage_two_is_stage_one_byte_for_byte() {
    let Some((one, two)) = both_stages("comparing the two stages needs one") else {
        return;
    };
    let (one, two) = (one.root.join(CLASSES), two.root.join(CLASSES));

    let classes = files_ending_in("class", &one, &[""]);

    assert!(!classes.is_empty(), "stage 1 holds no class file");
    assert_eq!(listing(&two), listing(&one), "stage 2 lists other classes");
    for class in classes {
        let first = fs::read(one.join(&class)).expect("a class of stage 1 is readable");
        let second = fs::read(two.join(&class)).expect("a class of stage 2 is readable");
        assert_eq!(
            first_difference(&first, &second),
            None,
            "{class}: the bytes first differ at this offset"
        );
    }
}

/// `docs/specs/executable-examples.md`: every program under `tests/spec` runs under the launcher
/// of stage 2 as under `lumen`.
#[test]
fn every_program_under_tests_spec_runs_under_stage_two_as_under_lumen() {
    let Some((_, two)) = both_stages("running a program needs one") else {
        return;
    };

    the_same_from_the_launcher(&two, &fixture_cases(&["run"], &["tests/spec"]));
}

/// Stage 1, which `lumen` builds, and stage 2, which the launcher of stage 1 builds, or nothing
/// where no JDK can run stage 1.
fn both_stages(needs: &str) -> Option<(Stage, Stage)> {
    let Some(_) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and {needs}");
        return None;
    };
    let one = Stage::built(LAUNCHED);
    let two = Stage::built_by(&one);
    Some((one, two))
}

/// A line of `path length` for each class file under `classes`, in the order the paths sort.
fn listing(classes: &Path) -> String {
    let mut listing = String::new();
    for class in files_ending_in("class", classes, &[""]) {
        let length = fs::metadata(classes.join(&class))
            .expect("a class file has a length")
            .len();
        let _ = writeln!(listing, "{class} {length}");
    }
    listing
}

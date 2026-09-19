//! The executable examples of `tests/spec/format/`: each `.unformatted` file and its `.lm`.
//!
//! That every example under `tests/spec/` compiles at all belongs to the harness in `crates/cli`,
//! which `docs/specs/executable-examples.md` describes.

use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use crate::common::formatted;

#[test]
fn every_unformatted_example_formats_to_the_file_beside_it() {
    let directory = spec_directory().join("format");
    let mut checked = 0;
    for path in read_dir_sorted(&directory) {
        if path.extension().is_none_or(|kind| kind != "unformatted") {
            continue;
        }
        let source = read_to_string(&path).expect("an example is readable");
        let canonical = read_to_string(path.with_extension("lm")).expect("a sibling `.lm`");
        assert_eq!(formatted(&source), canonical, "{}", path.display());
        checked += 1;
    }
    assert!(checked > 0, "tests/spec/format holds at least one example");
}

/// `tests/spec/`, reached from this crate.
fn spec_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/spec")
}

fn read_dir_sorted(directory: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", directory.display()))
        .map(|entry| entry.expect("a directory entry is readable").path())
        .collect();
    entries.sort();
    entries
}

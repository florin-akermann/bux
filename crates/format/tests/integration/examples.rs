//! The executable examples: the language's own files, held to canonical form.

use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use lumen_format::format;
use lumen_parser::parse;

use crate::common::formatted;

/// `tests/spec/`, reached from this crate.
fn spec_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/spec")
}

/// Every `.lm` file under `tests/spec/`, in a stable order.
fn examples() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(&spec_directory(), &mut found);
    found.sort();
    found
}

fn collect(directory: &Path, found: &mut Vec<PathBuf>) {
    let entries = read_dir_sorted(directory);
    for entry in entries {
        if entry.is_dir() {
            collect(&entry, found);
        } else if entry.extension().is_some_and(|kind| kind == "lm") {
            found.push(entry);
        }
    }
}

fn read_dir_sorted(directory: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", directory.display()))
        .map(|entry| entry.expect("a directory entry is readable").path())
        .collect();
    entries.sort();
    entries
}

#[test]
fn every_example_that_parses_is_already_in_canonical_form() {
    for path in examples() {
        let source = read_to_string(&path).expect("an example is readable");
        if parse(&source).is_err() {
            continue;
        }
        let canonical = format(&source).expect("a parsed example formats");
        assert_eq!(canonical, source, "{} is not canonical", path.display());
    }
}

#[test]
fn every_unformatted_example_formats_to_the_file_beside_it() {
    let directory = spec_directory().join("format");
    for path in read_dir_sorted(&directory) {
        if path.extension().is_none_or(|kind| kind != "unformatted") {
            continue;
        }
        let source = read_to_string(&path).expect("an example is readable");
        let canonical = read_to_string(path.with_extension("lm")).expect("a sibling `.lm`");
        assert_eq!(formatted(&source), canonical, "{}", path.display());
    }
}

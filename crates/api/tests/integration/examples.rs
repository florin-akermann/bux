//! The executable examples under `tests/spec/api/`: each `.lm` file has the page in its `.api`.
//!
//! `docs/specs/executable-examples.md` gives a phase a sibling file of the same stem to record
//! what it sees, and a page is what this phase sees.

use std::fs;
use std::path::{Path, PathBuf};

use crate::common::page;

#[test]
fn every_example_has_the_page_its_sibling_holds() {
    let sources = examples();
    assert!(
        !sources.is_empty(),
        "tests/spec/api holds at least one example"
    );
    for source_path in sources {
        let source = fs::read_to_string(&source_path).expect("an example is UTF-8");
        let expected_path = source_path.with_extension("api");
        let expected = fs::read_to_string(&expected_path)
            .unwrap_or_else(|error| panic!("{}: {error}", expected_path.display()));
        assert_eq!(page(&source), expected, "{}", source_path.display());
    }
}

/// Every example under `tests/spec/api/`, in a stable order.
fn examples() -> Vec<PathBuf> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/spec/api");
    let mut sources: Vec<PathBuf> = fs::read_dir(directory)
        .expect("tests/spec/api exists")
        .map(|entry| entry.expect("a directory entry is readable").path())
        .filter(|path| path.extension().is_some_and(|kind| kind == "lm"))
        .collect();
    sources.sort();
    sources
}

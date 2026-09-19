//! The executable examples under `tests/spec/parser/`.
//!
//! A `.lm` file has a sibling `.ast` file holding the tree it parses to, or a sibling `.error`
//! file holding the failure it is rejected with. A file with neither, or both, is a mistake in
//! the example itself and fails the run.

use std::fs;
use std::path::{Path, PathBuf};

use crate::common::{render, render_error};

fn examples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/spec/parser")
}

fn example_sources() -> Vec<PathBuf> {
    let mut sources: Vec<PathBuf> = fs::read_dir(examples_dir())
        .expect("tests/spec/parser exists")
        .map(|entry| entry.expect("directory entry is readable").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "lm"))
        .collect();
    sources.sort();
    sources
}

#[test]
fn every_example_parses_to_its_expected_tree_or_error() {
    let sources = example_sources();
    assert!(
        !sources.is_empty(),
        "tests/spec/parser holds at least one example"
    );
    for source_path in sources {
        check_example(&source_path);
    }
}

fn check_example(source_path: &Path) {
    let source = fs::read_to_string(source_path).expect("example is UTF-8");
    let tree_path = source_path.with_extension("ast");
    let error_path = source_path.with_extension("error");
    match (read_if_present(&tree_path), read_if_present(&error_path)) {
        (Some(expected), None) => {
            assert_eq!(render(&source), expected, "{}", source_path.display());
        }
        (None, Some(expected)) => {
            assert_eq!(render_error(&source), expected, "{}", source_path.display());
        }
        _ => panic!(
            "{} needs exactly one of {} and {}",
            source_path.display(),
            tree_path.display(),
            error_path.display()
        ),
    }
}

fn read_if_present(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

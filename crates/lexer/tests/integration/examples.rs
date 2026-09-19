//! The executable examples under `tests/spec/lexer/`: each `.lm` file lexes to its `.tokens` file.

use std::fs;
use std::path::{Path, PathBuf};

use crate::common::render;

fn examples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/spec/lexer")
}

fn example_sources() -> Vec<PathBuf> {
    let mut sources: Vec<PathBuf> = fs::read_dir(examples_dir())
        .expect("tests/spec/lexer exists")
        .map(|entry| entry.expect("directory entry is readable").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "lm"))
        .collect();
    sources.sort();
    sources
}

#[test]
fn every_example_lexes_to_its_expected_tokens() {
    let sources = example_sources();
    assert!(
        !sources.is_empty(),
        "tests/spec/lexer holds at least one example"
    );
    for source_path in sources {
        let source = fs::read_to_string(&source_path).expect("example is UTF-8");
        let expected_path = source_path.with_extension("tokens");
        let expected = fs::read_to_string(&expected_path).unwrap_or_else(|_| {
            panic!(
                "{} has no {}",
                source_path.display(),
                expected_path.display()
            )
        });
        assert_eq!(render(&source), expected, "{}", source_path.display());
    }
}

//! Helpers shared by the parser's behaviour tests.
//!
//! Reading a rendered tree beside its source is how a parser behaviour is stated here, and
//! `printed.rs` is where a tree is rendered.

use crate::printed::render;

/// The rendered tree of `source` below its first `above` lines, unindented by `depth` levels.
///
/// A test about one construct wraps it in the smallest item that can hold it, then reads only
/// the part it is about.
pub fn inner(source: &str, above: usize, depth: usize) -> Vec<String> {
    shape(source)
        .split_off(above)
        .iter()
        .map(|line| line[depth * 2..].to_owned())
        .collect()
}

/// The rendered tree with the spans dropped, for a behaviour that is about shape alone.
///
/// Spans are specified by the executable examples under `tests/spec/parser/`, whose `.ast`
/// files keep them, and by the tests that are about spans.
pub fn shape(source: &str) -> Vec<String> {
    render(source).lines().map(without_span).collect()
}

fn without_span(line: &str) -> String {
    match line.rsplit_once(' ') {
        Some((head, tail)) if is_span(tail) => head.to_owned(),
        _ => line.to_owned(),
    }
}

fn is_span(text: &str) -> bool {
    text.split_once("..").is_some_and(|(start, end)| {
        !start.is_empty()
            && !end.is_empty()
            && start.bytes().all(|byte| byte.is_ascii_digit())
            && end.bytes().all(|byte| byte.is_ascii_digit())
    })
}

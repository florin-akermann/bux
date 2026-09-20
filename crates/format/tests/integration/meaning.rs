//! Formatting never changes what a program says.
//!
//! The printer drops the parentheses the parser dropped and writes back only the ones the shape
//! needs, so this is the property that keeps it honest. Each source below is one the printer
//! could get wrong by writing a parenthesis too few or one too many.

use crate::common::{formatted, tree};

/// Expressions whose shape survives being written in canonical form.
const EXPRESSIONS: [&str; 24] = [
    "(a + b) * c",
    "a + (b * c)",
    "a - (b - c)",
    "(a - b) - c",
    "!(a && b)",
    "(!a) && b",
    "(a + b).c",
    "a.f(b)",
    "(a + b).f(c)",
    "(-a).b",
    "-a.b",
    "-(a + b)",
    "- 7",
    "-7.abs()",
    "--5",
    "!!a",
    "((a))",
    "f((a + b), c)",
    "-(7)",
    "-(-(7))",
    "-(7.abs())",
    "-(7).abs()",
    "-(7?)",
    "!(7.abs())",
];

/// Statements whose shape survives it too, including the headers that forbid a record literal.
const STATEMENTS: [&str; 6] = [
    "if (user { active: true }).active {\n    }",
    "for (user { active: true }).active {\n    }",
    "match (user { active: true }).active {\n        A => 1\n    }",
    "match a {\n        0 | 1 => 1\n        _ => 2\n    }",
    "a = (b + c) * d",
    "return (a + b) * c",
];

#[test]
fn formatting_an_expression_leaves_its_shape_alone() {
    for source in EXPRESSIONS {
        assert_same_shape(source);
    }
}

#[test]
fn formatting_a_statement_leaves_its_shape_alone() {
    for source in STATEMENTS {
        assert_same_shape(source);
    }
}

/// Asserts that `body`, inside a function, parses the same before and after formatting.
fn assert_same_shape(body: &str) {
    let source = format!("fn f() {{\n    {body}\n}}\n");
    let canonical = formatted(&source);
    assert_eq!(tree(&canonical), tree(&source), "{body}");
}

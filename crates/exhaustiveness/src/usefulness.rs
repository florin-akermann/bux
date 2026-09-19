//! The values a set of arms leaves uncovered.
//!
//! The arms become a matrix of one row each, which is then narrowed one column at a time: the
//! leading column is opened up under each constructor of its type, and whatever is left under one
//! of them is an answer. A column of a type with more values than a declaration writes down is
//! covered only by a name that binds. `docs/specs/exhaustiveness.md` states what that amounts to.

use crate::pattern::Pat;
use crate::space::{Signature, Space};

/// One arm, as the columns it matches; the whole matrix is one of these per arm.
type Row = Vec<Pat>;

/// Every value of `width` columns that no row of `rows` covers.
///
/// Each answer is itself a row: the patterns an arm would have to write to cover what is left.
pub(crate) fn uncovered(space: &Space, rows: &[Row], width: usize) -> Vec<Row> {
    let Some(narrower) = width.checked_sub(1) else {
        return if rows.is_empty() {
            vec![Vec::new()]
        } else {
            Vec::new()
        };
    };
    match leading_siblings(space, rows) {
        Some(siblings) => each_constructor(space, rows, narrower, siblings),
        None => anything_uncovered(space, rows, narrower),
    }
}

/// The constructors of the leading column's type, when a declaration writes them down.
fn leading_siblings<'a>(space: &'a Space, rows: &[Row]) -> Option<&'a [Signature]> {
    rows.iter()
        .find_map(|row| match &row[0] {
            Pat::Constructed { name, .. } => Some(name),
            Pat::Wildcard | Pat::Literal => None,
        })
        .and_then(|name| space.siblings(name))
}

/// What is left when the leading column has more values than a declaration writes down.
///
/// Only a name that binds covers such a column, so only the rows that write one say anything
/// about the columns after it, and what is left of the leading column is named `_`.
fn anything_uncovered(space: &Space, rows: &[Row], narrower: usize) -> Vec<Row> {
    let binding: Vec<Row> = rows
        .iter()
        .filter(|row| row[0] == Pat::Wildcard)
        .map(|row| row[1..].to_vec())
        .collect();
    uncovered(space, &binding, narrower)
        .iter()
        .map(|row| led_with(vec![Pat::Wildcard], row))
        .collect()
}

/// What is left under each constructor of the leading column.
///
/// A constructor no row writes has nothing narrowing it, so what is left under it is everything
/// it builds; one that some row writes is narrowed by what those rows write under it.
fn each_constructor(
    space: &Space,
    rows: &[Row],
    narrower: usize,
    siblings: &[Signature],
) -> Vec<Row> {
    let mut left = Vec::new();
    for one in siblings {
        let under = specialised(rows, one);
        for row in uncovered(space, &under, one.carries + narrower) {
            let (arguments, rest) = row.split_at(one.carries);
            let led = Pat::Constructed {
                name: one.name.clone(),
                arguments: arguments.to_vec(),
            };
            left.push(led_with(vec![led], rest));
        }
    }
    left
}

/// The rows that say something about a value `one` built, with what it carries opened up.
fn specialised(rows: &[Row], one: &Signature) -> Vec<Row> {
    let mut under = Vec::new();
    for row in rows {
        let opened = match &row[0] {
            Pat::Constructed { name, arguments } if *name == one.name => arguments.clone(),
            Pat::Wildcard => vec![Pat::Wildcard; one.carries],
            Pat::Constructed { .. } | Pat::Literal => continue,
        };
        under.push(led_with(opened, &row[1..]));
    }
    under
}

/// `leading` in front of `rest`, which is how a row grows back as the answer is put together.
fn led_with(leading: Vec<Pat>, rest: &[Pat]) -> Row {
    let mut row = leading;
    row.extend_from_slice(rest);
    row
}

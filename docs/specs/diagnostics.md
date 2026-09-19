# Diagnostics

Every refusal the compiler makes is a diagnostic: a code, a message, the source it points at, and
often a line saying what to do about it.
`docs/implementation.md` section 8 sets the voice; this spec sets the layout, the codes, and the
exit codes.
No diagnostic mentions the JVM, a class file, or a stack frame.

## Intent

A reader who sees a diagnostic learns three things at once: what is wrong, where, and what to do.
The code is the stable handle on the first of those.
A spec or a test cites `L0105`, never the prose, which is then free to be reworded.

A code is assigned once and never reused, even after the diagnostic it named is removed.
Every code has one home: the place that raises it, and one explanation beside it.

## The layout

A rendered diagnostic is the message, the location, the source it points at, and the help:

```text
error[L0105]: comparisons do not chain
  --> demo.lm:2:5

  2 |     a < b < c
    |     ^^^^^^^^^

help: compare twice and join the two with `&&`
```

The first line is `error[`, the code, `]: `, and the message.
A message is one line and never ends in a full stop.
The second line is two blanks, `--> `, and the file, the line, and the column joined by colons.
A blank line follows, then the source line, then the caret row.
A diagnostic that has something to advise ends with a blank line, `help: `, and that line.
Every rendering ends with a newline.

The source line and the caret row share a gutter: two blanks, the line number, and one blank.
The caret row writes blanks where the number goes, and both rows continue with `| `.
The line and the column are one-based, as an editor counts them.
The column counts characters rather than bytes.
A tab before the span is written back as a tab, so the two rows line up however wide a tab is.

Only the line the span starts on is shown, however far the span runs.
A span that starts past the end of the file is shown on its last line.
A span that ends on a later line carets the rest of its first line and says where it ends:

```text
  2 |     a <
    |     ^^^ this runs on to line 3
```

## The codes

A code is `L` and four digits, grouped by the phase that raises it.
The grammar writes `L01xx`, canonical form `L02xx`, name resolution `L03xx`, and inference `L04xx`.
Exhaustiveness writes `L05xx`.

Every number and every long form lives in `crates/diagnostics/src/code.rs`, which declares them
together so that neither can be added without the other.
The long form of a code is the file in `crates/diagnostics/src/explanations/` named after it.

The grammar raises these, in `crates/parser/src/error.rs`:

- `L0100` — the grammar expected one thing and the source wrote another.
- `L0101` — a string has no closing quote.
- `L0102` — a character is not part of the language.
- `L0103` — a number does not fit in a whole number.
- `L0104` — a backslash is followed by something that is not an escape.
- `L0105` — comparisons are chained.
- `L0106` — brackets nest deeper than the parser descends.
- `L0107` — something other than a name is written on the left of `=` or `+=`.

Canonical form raises these, in `crates/format/src/lib.rs`:

- `L0200` — the file is not in canonical form.
- `L0201` — an import is written after a declaration, or two imports are out of sort.

One code covers all three ways a file departs from whitespace form, because they are one problem
and `lumen fmt` is the one answer to it.
Order is its own code because `lumen fmt` is not the answer to it: where a declaration belongs is
the author's decision, so the compiler says where rather than moving it.

Name resolution raises these, in `crates/resolver/src/error.rs`:

- `L0300` — nothing in scope has this name.
- `L0301` — a module declares the same name twice.
- `L0302` — a declaration or a binding hides a name that is already in scope.
- `L0303` — a declaration is written above something that uses it.
- `L0304` — a name that is not a value, such as a function or a module, is written as one.
- `L0305` — an assignment names something other than a `var` binding.

Type inference raises these, in `crates/types/src/error.rs`:

- `L0400` — a type met a type it does not match.
- `L0401` — a call passes more or fewer arguments than the function takes.
- `L0402` — a field is reached that the type reached through does not have.
- `L0403` — a type would have to contain itself.
- `L0404` — a record is built without one of the fields it declares.
- `L0405` — a record is written with one of its fields given a value twice.
- `L0406` — `==` or `!=` is written between two values of a type that has no `Eq`.
- `L0407` — a division is written with a divisor of zero, which has no answer.

Exhaustiveness raises these, in `crates/exhaustiveness/src/error.rs`:

- `L0500` — a `match` leaves a value of the type it matches unanswered.
- `L0501` — a `match` lists its arms in an order the type does not declare its variants in.

## `lumen explain`

`lumen explain <code>` prints the long form of one code and exits `0`.
The long form says what the diagnostic means, why the language refuses it, and what to write
instead, at more length than a `help:` line has room for.
It lives beside the catalogue, one file per code, so a code and its explanation never drift.

A code the compiler cannot raise is reported on standard error, and the run exits `2`.

## Exit codes

A command exits `0` when it has nothing to refuse, `1` when it refuses the program it was given,
and `2` when it cannot do its job at all.
A file that cannot be read and a code that does not exist are both `2`: neither is about a
program.
Diagnostics go to standard error, and only what was asked for goes to standard output.

## Properties

These hold and are checked with property-based tests:

1. A rendering names a line and a column that lie inside the source.
2. A rendering opens with its code and ends with a newline, for any span at all.

A code without an explanation, and two codes written the same way, are unwriteable rather than
checked.
`crates/diagnostics/src/code.rs` declares each code, its number, and the file holding its long
form in one place, so none of the three can be added without the others.

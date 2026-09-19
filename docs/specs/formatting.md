# Formatting

The formatter prints the one canonical form of a program.
It is not a phase of compilation but a gate on it: `docs/design.md` section 13 makes source that
is not in canonical form a compile error.
This spec covers the version 0.1 surface of `docs/design.md` and `docs/implementation.md` section 9.

## Intent

There is exactly one canonical text for every program, and the printer is its definition.
`format(source)` is that text, and a file compiles only if `format(source) == source`, byte for
byte.
The formatter has no options, so no diff is ever a formatting diff and no two agents disagree
about layout.

Formatting preserves meaning: `parse(format(source))` equals `parse(source)`, always.
The printer therefore never reorders, adds, or drops anything the parse tree holds.
Sequence is not part of canonical form in version 0.1; Item 011 is where order joins it.

The formatter reads the source, not only the tree, because comments are not part of the tree.
`docs/specs/grammar.md` drops comment tokens before parsing, and the printer puts them back from
the token stream.

## The shape of canonical form

**Indentation is four spaces per level**, and there are no tabs.
No line ends with a blank, and every line ends with a `\n`, the last one included.
An empty file is empty: it has no lines at all.

**Exactly one blank line separates two top-level items**, and no blank line appears anywhere else.
The file neither begins nor ends with a blank line.

**Everything the grammar does not force onto several lines is written on one line.**
The grammar forces four things apart:
a block, a record type body, a variant list of two or more variants, and the body of a `match`.
Each of those writes one entry per line, because the grammar separates its entries by newlines.
Everything else — a call, a record literal, a parameter list, a match arm — is comma-separated
or has one part, and is written on one line however long it runs.

There is no maximum line length.
A length limit would need a rule for where to break every construct, and every such rule is a
second way to write the same program.

## Spacing

One space goes around every binary operator, and around `=`, `:=`, `+=`, `->`, `=>`, and `|`.
A prefix `!` or `-` is written against what it negates, as is a `?`, a `.`, and a call's `(`.
A `-` before something whose first character is a digit keeps its parentheses, as `-(7.abs())`.
The grammar reads a `-` before a number as part of it, so `-7.abs()` is a different program.
A `,` and a `:` are written against what precedes them and followed by one space.
Nothing is written inside `(` or `<` and their closing partners.
A record literal with fields is spaced inside its braces, as `User { id: id }`; an empty one is
`User {}`.

## The constructs

An **import** is `import name`.

A **record type** opens with `{` at the end of the `type` line, writes one `name: Type` per line
one level in, and closes with `}` alone at the level of the line that opened it.
A record type with no fields still spans those two lines.

A **variant list** of one variant is written on the `type` line without a bar, as
`type UserId = UserId(Int64)`.
A list of two or more writes `type Name =`, then one `| Variant` per line, one level in.
A variant's record payload opens on that variant's line and closes at that variant's level.

A **function** is `fn name<T>(a: A, b: B) -> R`, then its block.
A parameter whose type is left to inference is written as the bare name.
A result type is written only when the source writes one.

A **block** opens with `{` at the end of the line that heads it, writes one statement per line one
level in, and closes with `}` alone at that head line's level.
A block with no statements still spans those two lines.

An **`if`** writes `if condition {`, and an `else` or `else if` continues the line of the `}` it
follows, as `} else if other {`.

A **`match`** writes `match scrutinee {`, then one `pattern => body` per line one level in, then
`}`.

A **`for`** writes `for {`, `for condition {`, or `for name in iterable {`.

A **`return`** is `return` or `return value`; `break` and `continue` are written alone.

## Comments

A comment is alone on its line, at the indentation of the line below it.
Its text is written as the author wrote it, with any trailing blank taken off.

The printer writes a comment above the line it was written on.
A comment that already sits on its own line therefore stays where it is, and a comment written at
the end of a line moves to its own line directly above that one.
`total := 0 // start at nothing` becomes those two lines, in that order, and formatting it again
changes nothing.

A comment that follows the last entry of a block, a record body, a variant list, or a `match` is
written at the level of those entries, above the closing `}`.
A comment after the last item of a file is written at the left margin, after one blank line.

A comment written inside a construct that canonical form joins onto one line loses that line.
It is written below the joined line instead, above whatever the source wrote next.

That rule is total: every comment in the source appears exactly once in the output, in source
order, and no comment is ever dropped or duplicated.

## `lumen fmt` and `lumen check`

`lumen fmt <file>` rewrites the file in canonical form, and writes nothing when it already is.
`lumen check <file>` reports the first line that is not in canonical form and exits non-zero.

A file that does not parse is reported as the parse error, by both commands, and neither writes.
Exit codes are `0` for a file in canonical form, `1` for one that is not, and `2` for one that
cannot be read.
Item 004 is where these reports gain their `error[L0142]:` rendering; here they are plain lines.

A deviation names the line number, what the line says, and what canonical form writes there.
The line number is one-based, as an editor counts lines.
A line canonical form does not write at all is named as such, and a file whose lines all match
is reported for the way it ends, which is how a missing final newline reads.

## Executable examples

`tests/spec/format/<name>.lm` files are already in canonical form, and formatting one changes
nothing.
A sibling `<name>.unformatted` file, where there is one, formats to the `.lm` file beside it.
The format crate's integration tests walk that directory and name the failing example.

Every `.lm` file under `tests/spec/` that parses is in canonical form, and one test asserts it
of all of them.
The language's own examples are then the largest evidence that the printer is right.
A `.lm` file that does not parse is a compile-fail example, and canonical form says nothing about
one.

## Properties

These hold and are checked with property-based tests:

1. Formatting is idempotent: `format(format(source))` equals `format(source)`.
2. Formatted output parses.
3. Formatting preserves the tree: `parse(format(source))` equals `parse(source)`, up to spans.
4. Every comment of the source appears once in the output, in source order.
5. Formatting is deterministic.

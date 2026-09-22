# Executable examples

The files under `tests/spec/` are the language specification.
Prose can drift from the compiler; an example cannot, because the compiler is run against it.
`docs/implementation.md` section 7 asks for these examples; this spec says how they are written
and what running one amounts to.

## Intent

An example is a whole Lumen file that a reader can open and learn the language from.
It is not a fixture cut down to one phase, so nothing in `tests/spec/` is a fragment.
Either the compiler accepts the file, or the file says which diagnostic the compiler refuses it
with.

## File layout

An example is a `.lm` file under `tests/spec/<area>/`, where the area names a part of the language.
The areas that exist are the ones some example needs; none is created ahead of its first example.

A phase that wants to record more about an example puts it in a sibling file of the same stem.
`program.lm` and `program.tokens` are the same example seen by the lexer, and `program.ast` is
that example seen by the parser.
Only the `.lm` file is the example; a sibling is one phase's view of it.

## The expectation

An example with no header must compile.

An example the compiler refuses says so on its first line:

```text
// expect-error: L0105
```

The header is the word `expect-error`, a colon, and one diagnostic code from
`docs/specs/diagnostics.md`.
It cites the code and never the message, so rewording a diagnostic does not touch an example.
A code the compiler cannot raise fails the run, naming the file that wrote it.

An example that is run says so on its first line instead:

```text
// expect-run
// > 17 / 5 is Some(3)
// > 17 / 0 is None
```

It must compile, declare `main` at the shape `docs/specs/run.md` states, and run as it says.
It must also write exactly what the lines under the header say it writes.
Each line is the word `>` after the comment marker, a space, and one line the program writes.
They come directly under the header, in the order the program writes them.
A line the program writes empty is stated with the marker and nothing after it.
Canonical form keeps no space at the end of a line, which is why there is none to write.

A line opening with a marker anywhere else fails the run, naming the file.
Such a line is one an author meant to state and the harness would never have compared.
Reading it as a program that wrote the wrong thing would send the reader to the wrong file.
An example that is not run states nothing about a run at all, for the same reason.
An example that states none of them writes nothing at all, which is as much a claim as any other.

### What the program is run with

The header carries the words the program is run with, after the word `expect-run`:

```text
// expect-run --wide notes.txt
```

Each word is one argument, and the program is handed them in the order the header writes them.
A header with nothing after it runs the program with no argument at all.
An argument holding a space cannot be stated yet; the example needing one adds a form.

### What the program writes to standard error

A line the program writes to standard error is stated with `!` in place of `>`:

```text
// expect-run
// > the answer
// ! there was nothing to divide among
```

It is read exactly as a line of standard output is, and it is compared against standard error.
The two channels are compared apart, so a line stated on one and written on the other fails.
Lines of the two kinds may be written in any order under the header, because each channel is
compared only against the lines stated for it.

### What the program ends with

The status the program ends with is stated with the word `status` and that number:

```text
// expect-run
// status 3
```

An example stating no status says the program ends with `0`, which is as much a claim as any
other: a program with nothing to say gives back `0`.
A `status` line whose number is no whole number fails the run, naming the file, because it is a
line the author meant to state and the harness would not have compared.

An example states what it works out rather than asserting it in prose.
`17 / 5` being `Some(3)` is then something the harness checks, and something a reader can see.

An example may carry ordinary comments below its expectation.

## Compiling an example

An example compiles when `lumen check` accepts it.
Today that means it parses, is in canonical form already, and has a definition for every name.
Every phase the compiler grows joins that check, so an example that compiles today keeps having to
compile.

A refused example must be refused with exactly the code its header names.
The first diagnostic decides, because the compiler stops at it.
A refused example that parses is still in canonical form, so `L0200` is never what stops one.

## Running an example

An example headed `// expect-run` is run with `lumen run`, and with the words the header states.
It must end with the status the header states, which is `0` where the header states none.
`docs/specs/run.md` says what running amounts to and where the JDK comes from.

What the program writes to standard output is compared with what the header states.
What it writes to standard error is compared the same way, against the lines stated with `!`.
The stated lines are each followed by a line break, and the comparison is exact.
An output ending without a line break cannot be stated yet; the example needing it adds a form.

A mismatch names the example's path, what the header stated, and what the program wrote.
Both are shown as the text they are, so a line break or a space that differs is visible.

Running needs a JDK, and the test suite does not.
An example that is run is skipped when `JAVA_HOME` names none, and the skip says so by name, so a
run that proves less says which examples it did not reach.
Every other example is held to its expectation either way, because nothing else needs a JVM.

## The harness

One test binary walks `tests/spec/` and holds every example to its expectation.
It walks in a stable order so two runs report the same first failure.
A failure names the example's path, so the file to open is never in doubt.
An empty `tests/spec/` is itself a failure: the specification is never allowed to be nothing.

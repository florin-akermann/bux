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
```

It must compile, declare `main`, and run to the end without failing.
That is the whole expectation: version 0.1 gives a program no way to say anything else about how
it went, so an example claims nothing an example could not check.

An example may carry ordinary comments; only the first line is read as an expectation.

## Compiling an example

An example compiles when `lumen check` accepts it.
Today that means it parses, is in canonical form already, and has a definition for every name.
Every phase the compiler grows joins that check, so an example that compiles today keeps having to
compile.

A refused example must be refused with exactly the code its header names.
The first diagnostic decides, because the compiler stops at it.
A refused example that parses is still in canonical form, so `L0200` is never what stops one.

## Running an example

An example headed `// expect-run` is run with `lumen run`, and must end with status 0.
`docs/specs/run.md` says what running amounts to and where the JDK comes from.

Running needs a JDK, and the test suite does not.
An example that is run is skipped when `JAVA_HOME` names none, and the skip says so by name, so a
run that proves less says which examples it did not reach.
Every other example is held to its expectation either way, because nothing else needs a JVM.

## The harness

One test binary walks `tests/spec/` and holds every example to its expectation.
It walks in a stable order so two runs report the same first failure.
A failure names the example's path, so the file to open is never in doubt.
An empty `tests/spec/` is itself a failure: the specification is never allowed to be nothing.

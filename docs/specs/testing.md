# A test block, and a run of a package

## Intent

An example is one expression on one comment line, and `docs/specs/doc-examples.md` states it.
Some claims need more than one expression: they bind names, build a value in a loop, or call `io`.
Without a test block, such a claim becomes a function that only an example calls.
That function then needs an example of its own, and it ships in every program.
A `test` block states such a claim, and `bux test` runs it with the examples.
`docs/rationale.md` section 11 gives the reason, and this page states the behaviour.

## What a test is

A test is the word `test`, a name in quotes, and a block:

```text
test "a sum does not depend on the order" {
    var numbers = []
    for number in [4, 5, 6] {
        numbers = list.push(numbers, number)
    }
    summed(numbers) == summed([6, 5, 4])
}
```

The block is written as the body of a function, and its value is a `Bool`.
The test holds when the value is `true`, and it does not hold when the value is `false`.
The block can bind names, use every statement, and call `io`.
A test has no signature, no example, and no doc comment requirement.
Nothing calls a test except `bux test`, so a test has no name that code can use.

`test` is a keyword, as `docs/specs/lexer.md` states, so no name is spelled `test`.
`docs/specs/grammar.md` gives the rule `test = "test" String block`.

## Where a test is written

A test is an item, and it is written at the top level of a module.
A test written where a statement belongs is `L0110`, at the word `test`:

```text
error[L0110]: a test is written at the top level of a module, and this one is inside a body
```

The help is "a test is an item of its own; write it below every declaration of the module".

Tests come last, after every declaration, in the order the author writes them.
A declaration written below a test is `L0201`, as `docs/specs/formatting.md` states.
The message is "this declaration is written after a test".
The help is "tests come last, after every declaration: `bux fmt` moves it above them".
`bux fmt` moves such a declaration above the tests, and keeps the tests in the order written.

## The name of a test

The name is a string literal, and it can hold every escape a string can hold.
A report names a test by its name, so two tests of one module have two names.
Two tests of one module with the same name are `L0319`, at the name of the second:

```text
error[L0319]: two tests of this module are named "a"
```

The help is "a report names a test by its name; rename one of the two".
Two tests in two modules can have the same name, because a report also names the module.

## The type of the body

The body of a test is held to `Bool`, as the body of a function is held to its return type.
A body of another type is `L0400`, at the body.
Every other rule of a function body applies to the body of a test, such as exhaustiveness.

## What leaves a test out

`bux build` and `bux run` leave every test out, so a test never ships in a program.
`bux check` checks every test, so a test that is refused is refused while a reader writes it.
`bux api` shows no test, because a test is no part of the surface of a module.
A hole in a test does not stop `bux build`, because the build leaves the test out.
`bux test` refuses that hole as `L0600`, at its place in the test.

## What `bux test` runs

`bux test <file>` runs the examples and the tests of that module.
`bux test <dir>` runs every module of the package in that directory, in sorted name order.
`bux test` with no path runs the package in the current directory.
A directory with no `bux.package` is no package, and the run stops with exit code 2.

A run of a package does not stop at a module that fails.
It runs each module as `bux test <file>` runs it, and it reports what each module found.
Thus one run names every module that fails.

## A package runs on a pool

The modules of a package run at the same time, on one worker for each processor of the JVM.
A pool, which is a `process`, gives each module to the next worker that is free.
The pool keeps each answer by the number of its module in the sorted order.
So the report is in file order, and it is the report of the modules run one after another.
The exit code and each line are the same as in a run of one module after another.
A worker keeps a memo of the modules it typed, and it uses the memo for its next module.
So a module that many modules import is typed once for each worker, and not once for each module.
A JVM error that stops a worker, such as a stack overflow, stops the run with that error.

## How a run is put together

`docs/specs/doc-examples.md` states the module that a run writes for the examples.
A run writes each test into that module as a function `is_held_<k>`, with `k` from 0.
The function gives back a `Bool`, and its body is the block of the test, as the author wrote it.
The `main` of the run tries each example first, then each test, in the order they are written.
A test that did not hold writes the line `bux: test <k>`, and the run reads that line back.
A module that declares a name `is_held_<k>` leaves the run no room, and `L0604` says so.

A refusal of the run inside a test body is reported against the line in the original file.
A module that states no example and no test has nothing to run, and it is a run that held.

## The report

A run passes on each line that its tests write on standard output, in the order written.
So a test can say why it did not hold, or which check it skipped and why.
The lines that mark an example or a test that did not hold are read, and not passed on.
A run in which every example and every test held, and no test wrote a line, prints nothing.
An example that did not hold is `L0603`, as `docs/specs/doc-examples.md` states.
A test that did not hold is one line with the module, the line of the word `test`, and the name:

```text
sums.bx:14: test "a sum does not depend on the order" did not hold
test: 1 tests, 3 examples; 1 failed
```

The last line follows the report when something did not hold, and a run writes it once.
It counts the tests and the examples that the run tried, and the ones that did not hold.
A module that the compiler refuses is reported as `bux check` reports it.

```text
0  every example and every test held
1  a module was refused, or an example or a test did not hold
2  the run could not be started, or the path is no file and no package
```

## The tests of the compiler

The compiler's tests are the tests of `tests/`, and `bin/bux test tests` runs them all.
`.githooks/pre-commit` runs it after `bin/bootstrap`, and a commit needs status 0 from it.
A test there calls a check of its module, which gives back what it tried and what failed.
The test writes a line for each failure and a line `skipped: <why>` for each check it skipped.
A test that skips a check holds, so the reason is on standard output and the run goes on.
A check that starts a JVM stops it after 60 s, and a JVM stopped so is a failure of the check.

`tests/documented.bx` runs the examples and the tests of `library/`, `compiler/`, and `tests/spec/`.
It leaves out the tests under `tests/spec/`, because each of those files states its own answer.
It leaves out `tests/` itself, because `bin/bux test tests` runs those modules directly.
It skips `library/prelude.bx`, because the run of its examples declares `or` a second time.
The golden answers in `tests/commands/fixtures.txt` hold `bux test` on `tests/spec/testing/`.
`bin/bux run tests/golden.bx` writes each golden file under `tests/commands/` again.

## The errors

| name                   | code    | message                                                   |
|------------------------|---------|-----------------------------------------------------------|
| a test inside a body   | `L0110` | a test is written at the top level of a module, ...       |
| two tests, one name    | `L0319` | two tests of this module are named "a"                    |
| a declaration after it | `L0201` | this declaration is written after a test                  |
| a body not `Bool`      | `L0400` | expected `Bool`, found `Int`                              |

## Properties

These hold and are checked by drawn properties, each a test of `tests/`:

1. The word `test` is read as a keyword, and a longer name that opens with it is a name.
2. A test written among the statements of a body is refused as `L0110`, at the word `test`.
3. A declaration written below a test is `L0201`, and a test written last is in place.
4. A drawn package on a pool of drawn size reports what its modules report when run in turn.
   `tests/packaged.bx` holds it, and each module of a drawn package has a drawn outcome.
   The example of a module holds or fails, the compiler refuses the module, or it states nothing.

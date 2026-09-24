# A test block, and a run of a package

## Intent

An example is one expression on one comment line, and `docs/specs/doc-examples.md` states it.
Some claims need more than one expression: they bind names, build a value in a loop, or call `io`.
Without a test block, such a claim becomes a function that only an example calls.
That function then needs an example of its own, and it ships in every program.
A `test` block states such a claim, and `bux test` runs it with the examples.
`docs/design.md` section 11 gives the reason, and this page states the behaviour.

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
The help is "tests come last, after every declaration".
`bux fmt` keeps the tests in the order the author wrote them.

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

A run in which every example and every test held prints nothing.
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

## The runner

`tests/documented.bx` runs the tests of every module it runs the examples of.
It leaves out the tests under `tests/spec/`, because each of those files states its own answer.
The golden answers in `tests/commands/fixtures.txt` hold `bux test` on `tests/spec/testing/`.
The summary line of the runner counts the tests that it ran.

## The errors

| name                   | code    | message                                                   |
|------------------------|---------|-----------------------------------------------------------|
| a test inside a body   | `L0110` | a test is written at the top level of a module, ...       |
| two tests, one name    | `L0319` | two tests of this module are named "a"                    |
| a declaration after it | `L0201` | this declaration is written after a test                  |
| a body not `Bool`      | `L0400` | expected `Bool`, found `Int`                              |

## Properties

These hold and are checked by drawn properties in the runner:

1. The word `test` is read as a keyword, and a longer name that opens with it is a name.
2. A test written among the statements of a body is refused as `L0110`, at the word `test`.
3. A declaration written below a test is `L0201`, and a test written last is in place.

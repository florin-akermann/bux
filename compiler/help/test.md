Run the examples and the tests of a module, or of every module of a package.

Every public function a module declares at the top level states at least one example, and
`bux test` runs them. A `private` function may state one too. An example is a line of the
comment above the function, and it is Bux rather than prose:

```text
// Divides `total` among `people`, giving back nothing where there is nobody to divide among.
//
// example: or(shared(total: 17, people: 5), 0) == 3
// example: or(shared(total: 17, people: 0), 0) == 0
fn shared(total: Int, people: Int) -> Option<Int> {
    total / people
}
```

The line is the comment marker, the word `example`, a colon, and one expression of type `Bool`
that is `true` when it runs. It is written in the module that declares the function, so it reaches
every name that module has. A signature says what a function takes and gives back, and says
nothing about what it does; an example says that, and says it so the compiler can hold the
function to it.

`main` states none, because it is reached by running the module rather than by calling it, and
running the module is its example. An example written anywhere else — inside a body, above a type,
above `main`, or above nothing at all — is refused rather than skipped, because a marker the run
would silently pass over is a claim nobody would ever check.

A claim that needs more than one expression is a test. A test is the word `test`, a name in
quotes, and a block whose value is a `Bool`:

```text
test "a sum does not depend on the order" {
    var numbers = []
    for number in [4, 5, 6] {
        numbers = list.push(numbers, number)
    }
    summed(numbers) == summed([6, 5, 4])
}
```

The block binds names, loops, and calls `io` as the body of a function does. A test is written at
the top level, after every declaration of its module; one inside a body is `L0110`, and a
declaration below a test is `L0201`. Two tests of one module with the same name are `L0319`, and a
block whose value is not a `Bool` is `L0400`. A test has no signature and no example, and
`bux build` and `bux run` leave every test out, so a test never ships in a program.

The examples and the tests are run as a module, because a module is the only thing there is to
run. `bux test` writes one out of what it was given: the module's imports, a `main` that tries
each example and then each test in turn, and every declaration the module makes other than its
own `main`. That module is compiled and started the way any other is, its class files go into a
directory made for that run alone so nothing `bux build` wrote is touched, and a refusal of it is
reported against the line in the original file. Every module it imports is compiled into that
directory too, so an example reaching a name through an import runs exactly as the module does.

The run says which example or test did not hold by writing a line, and every line it writes opens
with a mark of its own, so a line the program writes for itself is never read as a report. Writing
a line is what `io.println` is for, so the module the run writes reaches `io`; a module that
declares `io` of its own leaves the run no room, and `L0604` says so. Only `bux test` is affected,
because only `bux test` writes a module.

Each other line the run writes is passed on to standard output, in the order written, so a test
can say why it did not hold. A run in which every example and every test held, and nothing wrote
a line, prints nothing, as every command that found nothing to report prints nothing. An example
that did not hold is reported as `L0603`, where it is written. A test that did not hold is
reported on one line with the module, the line of the word `test`, and the name:

```text
sums.bx:14: test "a sum does not depend on the order" did not hold
test: 1 tests, 3 examples; 1 failed
```

The last line counts what the module ran and what did not hold. Every example and every test that
did not hold is reported rather than the first. A module that states no example and no test has
nothing to run, and a module of types alone is such a module.

Handed a directory, `bux test` runs every module of the package, one module on each processor at
a time. It reports them in the order their names sort, as a run of one module after another would.
It does not stop at a module that fails, so one run reports every module that fails. With no path,
it runs the package in the current directory.

Running needs a JDK, which comes from `JAVA_HOME` and from nowhere else, exactly as `bux run`
takes it.

Exit codes: 0 when every example and every test held, 1 when the compiler refuses a module or an
example or a test did not hold, and 2 when the examples could not be run at all.

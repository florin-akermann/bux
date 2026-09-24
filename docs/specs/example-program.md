# The example program

## Intent

`example/` holds one Bux program that somebody would actually write.

Everything else in the tree is evidence about the compiler.
`tests/spec/` holds files that exist to be accepted or refused, and `src/` holds the compiler.
Neither is a program, and a reader who looks for what Bux is like has nowhere to look.

`example/` is that place.
It is read in two screens, it is everyday Bux, and the suite runs it, so it cannot rot.
It is a package, so a newcomer also reads the layout of a package in the first program they read.

## What the directory holds

`example/` is a package in the layout that `docs/specs/packages.md` states:

- `example/bux.package` is the manifest, and it names the package `example`.
- `example/src/main.bx` is the program, and it declares `main` and nothing else.
- `example/src/orders.bx` holds the types and the functions that `main` works with.
- `example/tests/ordered.bx` holds one `test` block, which reaches `orders` by its name.

No module imports `main`, so what a test reaches is in `orders`, and not in `main`.
The test module is not called `orders.bx`, because one name has one definition in a package.
A test module and a module of `src/` of one name are `L0317`.

Each module is a module like any other.
`bux check` holds it to canonical form and to every rule of the compiler.
The pre-commit sweep holds it to the same bar as the rest of the tree.
It is not an executable example, so it carries no `// expect-` header.
So `docs/specs/executable-examples.md` does not reach it.

A build writes class files under `example/target/`, beside the manifest, and none beside a source.
Those are output, not source, and git ignores `target/`.

## What the program is

The program shows the four things everyday Bux is made of:

- a record, `Basket`, with two fields
- an algebraic data type, `Order`, with a variant that carries a value and one that does not
- a `match` over that type, with one arm per variant, in the order the type declares them
- a `for … in` loop over a written list, which is what everyday Bux is mostly made of

It also uses `or`, so a reader meets `Option` in the one place version 0.1 hands them one.
It imports `io` and writes each answer, so a reader meets the one way a program shows anything.

Every function but `main` states an example, which `docs/specs/doc-examples.md` requires.
A reader therefore meets a signature and what it works out on the same screen.
`bux test` holds the package to both.

`main` takes a `List<String>` and gives back an `Int`, which `docs/specs/run.md` requires of it.
It reads no argument and gives back `0`, because it writes everything that it works out.

## What running it does

```sh
bux run example/src/main.bx
```

The program writes three lines and exits `0`:

```text
held: 7
each: 3
most: 6
```

Every answer that `main` works out is written, and nothing else is.
`io.println` writes each line, which `docs/specs/io.md` states.
`shown` gives a number its text, which the `Show<Int>` instance of the prelude is.
A reader therefore sees what the program worked out, not only that it reached the end.

No Bux program can fail at runtime, which `docs/specs/arithmetic.md` explains.
So a run that writes those three lines is the only run there is.

## What is tested

`tests/started.bx` runs `example/src/main.bx` through the command line.
It asserts that the program writes those three lines and exits `0`.
It then runs `bux test` on the package, and asserts the same exit.
That run holds the examples of `src/` and the test of `tests/`.

Each check is skipped with a named reason when `JAVA_HOME` names no JDK, as every run is.
The run happens on a copy outside the repository, so a test never writes into `example/`.

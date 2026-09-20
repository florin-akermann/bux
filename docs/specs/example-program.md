# The example program

## Intent

`example/` holds one Lumen program that somebody would actually write.

Everything else in the tree is evidence about the compiler.
`tests/spec/` holds files that exist to be accepted or refused, and `crates/` holds the compiler.
Neither is a program, and a reader looking for what Lumen is like has nowhere to look.

`example/main.lm` is that place.
It is read in one screen, it is everyday Lumen, and the suite runs it, so it cannot rot.

## What the directory holds

`example/` holds exactly one source file, `example/main.lm`.

It is a module like any other: `lumen check` holds it to canonical form and to every rule the
compiler has, and the pre-commit sweep holds it to the same bar as the rest of the tree.
It is not an executable example, so it carries no `// expect-` header and
`docs/specs/executable-examples.md` does not reach it.

A build writes class files beside the source.
Those are output, not source, and git ignores them.

## What the program is

The program shows the four things everyday Lumen is made of:

- a record, `Basket`, with two fields
- an algebraic data type, `Order`, with a variant that carries a value and one that does not
- a `match` over that type, with one arm per variant, in the order the type declares them
- a `for` loop, which is what everyday Lumen is mostly made of

It also uses `or`, so a reader meets `Option` in the one place version 0.1 hands them one.

Every function but `main` states an example, which `docs/specs/doc-examples.md` requires of one.
A reader therefore meets a signature and what it works out on the same screen, and `lumen test`
holds the file to both.

`main` returns `()` and takes nothing, which is what `docs/specs/run.md` requires of it.

## What running it does

```sh
lumen run example/main.lm
```

The program ends normally and exits `0`.

That is the whole of what it shows, and it is deliberate.
No Lumen program can fail at runtime, which `docs/specs/arithmetic.md` explains.
The answer this program works out is an `Int`, and version 0.1 has no way to write a number out.
`io.println` writes a line, which `docs/specs/io.md` states, but there is no text to hand it.
So a run that reaches the end is what this program shows, and the answer itself waits on that.

## What it cannot show yet

`for … in` over a `List` is the one shape of loop the file does not write.

`List<T>` is opaque: `docs/specs/types.md` states that version 0.1 has no syntax that builds one,
only `for` that walks one.
A function taking one is a function nothing can call, so the only example it could state is the
one `docs/specs/doc-examples.md` keeps for that case, which says nothing about the function.
The first program a reader meets should not teach that, so `fullest` counts its baskets rather
than walking a list of them, and the `for … in` form waits on a way to build a `List` to walk.

`main` binds names it does not use: there is nowhere for an answer to go, because version 0.1 has
no way to write a number out.

## What is tested

One test runs `example/main.lm` through the CLI and asserts it exits `0`.
Another runs its examples through `lumen test` and asserts the same.

It is skipped with a named reason when `JAVA_HOME` names no JDK, as every run in the suite is.
The run happens on a copy outside the repository, so a test never writes into `example/`.

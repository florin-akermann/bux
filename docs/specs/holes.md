# Holes

## Intent

An unfinished body says so in the language, rather than in a comment or in plausible wrong code.

`todo("a reason")` is that.
It stands where a value belongs, and takes whatever type is expected there.

A hole is accepted by `bux check` and refused by `bux build`.
Unfinished work is then something a compiler reports, rather than something a reader has to find.

`docs/design.md` section 5 states the rule; this spec states what the two commands do.

## What a hole is

`todo` is a function the prelude supplies:

```text
todo(reason: String) -> T
```

It is generic in what it gives back, so a hole fits wherever a value fits:

```text
fn total(users: List<User>) -> Int {
    todo("count them once the walk is written")
}
```

`T` is settled by the context, exactly as the type of `None` is.
A hole where a `String` belongs is a `String`, and one where a `List<User>` belongs is that.

It is an ordinary prelude name, not a keyword.
Bux hides no name, so a module that declares its own `todo` is refused with `L0302`, and a
binding or a parameter of that name is refused the same way.
`todo` therefore means the hole wherever it is in scope, which is everywhere.

A hole reads every name in scope where it is written, so `L0321` never refuses a name above it.

## What `bux check` does

Nothing.
A hole is well typed, so `bux check` accepts a module holding one and exits `0`.

That is the point: a file with a hole in it still has its names resolved, its types inferred, and
its `match`es checked, so the work around the hole is held to the same bar as finished work.

## What `bux build` does

It refuses, and it names every hole rather than the first:

```text
error[L0600]: this hole is not compiled
  --> demo.bx:2:5

  2 |     todo("count them once the walk is written")
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

help: `bux check` accepts a hole; fill it in, or build a module that holds none
```

The reason is written where the hole is, so the rendered line carries it.
One block is printed per hole, in the order they are written, with a blank line between two of
them, and the command exits `1`.

Every hole is named because a build is how a reader learns what is left, and stopping at the
first would make that a list of one.
This is the one refusal the compiler does not stop at the first of.

`bux run` builds before it runs, so it refuses the same way.

Nothing is written when a module holds a hole: the refusal comes before any class file.

## Why the build and not the check

`bux check` is what a reader runs while writing, and unfinished work is the state it is written
in.
A check that refused a hole would make the hole useless, and the reader would write the plausible
wrong code the hole exists to replace.

`bux build` is what produces something that runs.
A hole has nothing to run, so there is nothing for it to produce.

## The errors

| name       | code    | message                    |
|------------|---------|----------------------------|
| hole built | `L0600` | this hole is not compiled  |

`L0600` is raised by `bux build` and `bux run`, never by `bux check` or `bux fmt`.

A hole given something that is not a `String` is `L0400`, as any other mismatch is.
A hole given the wrong number of arguments is `L0401`, for the same reason.

## The hole listing written in Bux

`src/holes.bx` finds the holes in Bux.
It reads each module that `src/exhaustiveness.bx` accepts, in the order they load.
A call is a hole when resolution says that its callee is the `todo` of the prelude.

`holes.of_module(resolved)` gives every hole of one module, in the order they are written.
The code, the message, and the help of each hole are the ones that `bux build` shows.

`src/command.bx` renders the line and the column of each hole from the span.

`tests/unfinished.bx` holds the holes to the properties below, on modules with drawn holes.
The `holes_refused` check of `tests/enacted.bx` holds `bux build` to its refusal.

## Properties

These hold and are checked by drawn properties, each a test of `tests/`:

1. A module that holds no hole gives back no hole, whatever it is written out of.
2. Every hole a module holds is found, wherever in the module it is written.
3. A hole is a hole whatever type it is written in, because a hole takes the type it is given.

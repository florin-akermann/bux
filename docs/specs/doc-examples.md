# An example on every function

## Intent

A signature says what a function takes and gives back, and says nothing about what it does.
An example says that, and says it so that the compiler can hold the function to it.

`docs/design.md` section 11 requires one of every function a module declares.
Prose about a function is a claim nobody checks, and it drifts the moment the body changes.
An example is the same claim written as Bux, so `bux test` runs it and a wrong one is reported.

## What an example is

An example is one line of the comment above a function:

```text
// Divides `total` among `people`, giving back nothing where there is nobody to divide among.
//
// example: or(shared(total: 17, people: 5), 0) == 3
// example: or(shared(total: 17, people: 0), 0) == 0
fn shared(total: Int, people: Int) -> Option<Int> {
    total / people
}
```

The line is the comment marker, the word `example`, a colon, and one Bux expression.
The expression has the type `Bool`, and it is `true` when it runs.

It is written in the module that declares the function, so it reaches every name that module has.
It is one line, because the language writes an expression on one line and a comment is one line.

The marker is the run of slashes the comment opens with, so `///` states an example as `//` does.
A line an author meant as an example and a run passed over in silence would be the one claim
nobody ever checks, which is what this page exists to remove.

A function states as many examples as it has cases worth stating, one per line.
`docs/specs/formatting.md` leaves the text of a comment as the author wrote it, so nothing
rewrites a marker written without the space this page writes it with.

## An example need not call what it is written above

The rule is that a function states an example, and not that the example reaches it.

A compiler can check that an example is stated and that it holds, and it cannot check that the
example is a good one.
Requiring the name to appear would be a proxy for that, and it would refuse code that is right.

## The comment a function carries

The comment above a declaration is the unbroken run of comment lines directly above it.
A blank line between the run and the declaration ends it, and what is above that blank line
belongs to no declaration.

A run of comment lines that no declaration follows belongs to no declaration either.
That is where the header of an example file under `tests/spec/` sits, which is why an example
file's header is never read as documentation.

## Which functions carry one

Every function a module declares at the top level carries at least one example, except `main` and
one whose signature gives nothing back.
`docs/specs/api-surface.md` states that every such name is public, so every one of them is a name
another module will reach for.

`main` is exempt because it is reached by running the module rather than by calling it.
Running the module is the example of `main`.

A function written `-> ()` is exempt because an example is an expression that is true, and no
expression over a call giving nothing back is one.
`io.println` is such a function: what it does is write a line, and there is nothing to say about
the value it gives back, because it gives none.
A function whose result is left to inference is not exempt, because the signature is what a
reader of the declaration has, and it did not say.

## What refuses, and when

`bux check` accepts a module whose functions state no example.
`bux build`, `bux run`, and `bux test` refuse it.

That is the line `docs/specs/holes.md` draws for a hole, drawn here for the same reason.
An example calls the function it documents, so the function is written first, and a check that
refused a function without one would refuse the state every function passes through.
`bux check` is what a reader runs while writing.
`bux build` is what produces something that runs, and an undocumented function is unfinished
work the way a hole is.

Every undocumented function is named, and not only the first, because a build is how a reader
learns what is left.

```text
error[L0601]: `shared` states no example
  --> demo.bx:1:4

  1 | fn shared(total: Int, people: Int) -> Option<Int> {
    |    ^^^^^^

help: write `// example: <an expression that is true>` in the comment above it
```

## An example written where nothing carries one

An example line is refused wherever it is not in the comment above a function that carries one:

```text
error[L0602]: this example documents nothing
  --> demo.bx:7:5

  7 |     // example: or(shared(total: 17, people: 5), 0) == 3
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

help: an example goes in the comment above the function it documents
```

Inside a body, above a type, above `main`, and above nothing at all are each such a place.
A line written there is one an author meant to state and no run would ever have checked, and
reading it as a claim that holds would be reading a claim nobody made.
`docs/specs/executable-examples.md` refuses a stray `>` marker for the same reason.

## Running them

`bux test <file>` runs every example the module states, and then every test.
`docs/specs/testing.md` states a test, the run of a package, and the report of a test.

It runs the front end first and refuses what `bux check` refuses, with the same diagnostic.
It then holds the module to this page, and refuses `L0601` and `L0602` as a build does.

An example that does not hold is reported where it is written:

```text
error[L0603]: the example of `shared` does not hold
  --> demo.bx:3:1

  3 | // example: or(shared(total: 17, people: 5), 0) == 4
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

help: an example states what a function works out, and this one states what it does not
```

Every example that does not hold is reported, and not only the first.
A run in which every example held prints nothing, as every command that found nothing to report
prints nothing.

A module that states no example has nothing to run and is a run that held.
A module of types alone is such a module, and so is one that declares `main` and nothing else.

Running needs a JDK, which `docs/specs/run.md` says is the one `JAVA_HOME` names.

```text
0  every example the module states held
1  the module was refused, or an example did not hold
2  the examples could not be run at all
```

## How a run is put together

The examples run as a module, because a module is the only thing there is to run.

The run writes one: the module's own imports, then a `main` that tries each example in turn, then
every declaration the module makes other than `main`.
Each example is tried by an `if` that writes a line naming it when the example is not `true`.

Every line the run writes opens with a mark of its own, and a line without that mark is the
program talking.
`io.println` is the one way there is to write a line, so an example is free to write one too, and
a run reading bare names back would read an example's own output as a report about itself.
What the run reads back off standard output is the marked lines, which are exactly the examples
that did not hold.
The module's own `main` is not run: the examples are what `bux test` runs, and a module is put
through `bux run` to run its program.

The written module is compiled and run the way any other is, and its class files are written
into a directory made for that run alone, so nothing `bux build` wrote is touched and nothing
a run left behind is ever picked up.
The directory is taken away again whether the examples held or not.

It faces every rule except canonical form.
Canonical form is a rule about what an author writes, and `docs/specs/formatting.md` leaves the
text of a comment alone, so holding a module written around a comment to it would hold the author
to a form nothing spells out and `bux fmt` cannot repair.
The file the author wrote faced that rule already, on its way in.

A refusal of the written module is a refusal of the example it was written around.
The run knows which stretch of what it wrote came from where, so the refusal is reported against
the line in the original file, and no reader is ever shown text they did not write.
That is why an example that is not a `Bool`, or that calls something that is not there, reads as
a refusal of that example.

## What the run reaches for

The module a run writes reaches `io`, because writing a line is how it says which example did not
hold and `docs/specs/io.md` gives `io.println` as the one way to write one.

A module declares each name once, so a module that declares `io` of its own leaves the run no
room, and `L0604` says so before anything is compiled.
`bux check`, `bux build`, and `bux run` take such a module as they always did: it is the
run, and only the run, that has no room.

The `main` the run writes takes the words a program is run with, under the name `arguments`.
A module that declares `arguments` at the top level leaves the run no room for that name either,
and `L0604` says so the same way.

## The errors

| name                    | code    | message                             |
|-------------------------|---------|-------------------------------------|
| no example              | `L0601` | `x` states no example               |
| example documents none  | `L0602` | this example documents nothing      |
| example does not hold   | `L0603` | the example of `x` does not hold    |
| the run reaches the name| `L0604` | a run of the examples reaches `io`, and this module declares it |

`L0601` and `L0602` are raised by `bux build`, `bux run`, and `bux test`, and never by
`bux check` or `bux fmt`.
`L0603` is raised by `bux test` alone, because it is the only command that runs an example.

## The examples written in Bux

`compiler/command.bx` finds the examples of a module in Bux.
A `build`, a `run`, and a `test` refuse `L0601` and `L0602` with it, before the module is lowered.
`test` writes the run as this page says.
It puts a refusal of the run back into the file the author wrote.
It reads the marked lines back, and it gives `L0603` for each example that did not hold.

`tests/documented.bx` runs every `// example:` line of every module, as `bux test` runs it.
It writes and starts each run as `bux test` does, each in a JVM of its own.
`tests/commanded.bx` holds `test` to golden answers on each example of `tests/spec` and `library/`.

## Properties

These hold and are checked by drawn properties in the runner:

1. Every example a module states is found, wherever in the module it is written.
2. A module whose every function states an example is accepted, however many they state.
3. The text of an example is the text of the line it is written on, after the marker.
4. A span an example run reports is a span of the file the examples were read out of.

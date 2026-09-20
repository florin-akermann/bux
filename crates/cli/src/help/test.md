Run the examples a module states about its functions.

Every function a module declares at the top level states at least one example, and `lumen test`
runs them. An example is a line of the comment above the function, and it is Lumen rather than
prose:

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

The examples are run as a module, because a module is the only thing there is to run. `lumen test`
writes one out of what it was given: the module's imports, a `main` that tries each example in
turn, and every declaration the module makes other than its own `main`. That module is compiled
and started the way any other is, its class files go into a directory made for that run alone so
nothing `lumen build` wrote is touched, and a refusal of it is reported against the line in the
original file.

The run says which example did not hold by writing a line, and every line it writes opens with a
mark of its own, so a line the program writes for itself is never read as a report. Writing a line
is what `io.println` is for, so the module the run writes reaches `io`; a module that declares `io`
of its own leaves the run no room, and `L0604` says so. Only `lumen test` is affected, because only
`lumen test` writes a module.

A run in which every example held prints nothing, as every command that found nothing to report
prints nothing. An example that did not hold is reported as `L0603`, where it is written, and
every one that did not hold is reported rather than the first. A module that states no example has
nothing to run, and a module of types alone is such a module.

Running needs a JDK, which comes from `JAVA_HOME` and from nowhere else, exactly as `lumen run`
takes it.

Exit codes: 0 when every example the module states held, 1 when the compiler refuses the module or
an example did not hold, and 2 when the examples could not be run at all.

Report the first thing about a source file that the compiler will not have.

`lumen check` runs the front end over a file and stops at the first refusal. It reads the file,
holds it to canonical form, parses it, resolves every name in it, gives every expression a type,
and checks that every `match` answers for every value it may meet. Nothing is written back.

Canonical form comes first: a file that differs is reported with the line and column it is
about, that line under a row of carets, and a `help:` line naming the text canonical form writes
there. `lumen fmt` is the command that fixes it.

Name resolution comes next. A name with no definition, a module that declares one name twice, and
a binding that hides a name already in scope are each refused, because in Lumen one name has one
definition.

Type inference comes next. A type written where another is needed, a call with the wrong number
of arguments, a field a record does not declare, and a record built without one of its fields are
each refused. Inside a function the types are inferred, so a signature is written where it
documents a boundary rather than on every line.

Exhaustiveness comes last. A `match` that leaves a value of the type it matches unanswered is
refused, and the refusal names a value it does not cover. `lumen explain` says more about any code
that is printed.

Exit codes: 0 when the compiler has nothing to say, 1 when it refuses the program, and 2 when the
file cannot be read.

Report the first thing about a source file that the compiler will not have.

`lumen check` runs the front end over a file and stops at the first refusal. It reads the file,
holds it to canonical form, parses it, resolves every name in it, gives every expression a type,
and checks that every `match` answers for every value it may meet. Nothing is written back.

Canonical form comes first: a file that differs is reported with the line and column it is
about, that line under a row of carets, and a `help:` line naming the text canonical form writes
there. `lumen fmt` is the command that fixes it. An import written after a declaration, or two
imports out of sort, is reported here too, and `lumen fmt` does not fix that one: where an import
belongs is said, never rewritten.

Name resolution comes next. A name with no definition, a module that declares one name twice, and
a binding that hides a name already in scope are each refused, because in Lumen one name has one
definition. A declaration written above something that uses it is refused here as well: a file
reads top down, so the reader meets the intent before the detail.

Type inference comes next. A type written where another is needed, a call with the wrong number
of arguments, a field a record does not declare, and a record built without one of its fields are
each refused. Inside a function the types are inferred, so a signature is written where it
documents a boundary rather than on every line.

Exhaustiveness comes last. A `match` that leaves a value of the type it matches unanswered is
refused, and the refusal names a value it does not cover. A `match` that answers for everything
but lists its arms in an order the type does not declare its variants in is refused too, so a new
variant has exactly one place to be handled. `lumen explain` says more about any code that is
printed.

A hole is accepted. `todo("a reason")` stands where a value belongs and takes whatever type is
expected there, so an unfinished body is still resolved, typed, and checked like finished work.
`lumen build` is what refuses a hole, so this is the command to run while one is still there.

Exit codes: 0 when the compiler has nothing to say, 1 when it refuses the program, and 2 when the
file cannot be read.

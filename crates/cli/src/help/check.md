Report the first thing about a source file that the compiler will not have.

`lumen check` runs the front end over a file and stops at the first refusal. It reads the file,
holds it to canonical form, parses it, and resolves every name in it. Nothing is written back.

Canonical form comes first: a file that differs is reported with the line and column it is
about, that line under a row of carets, and a `help:` line naming the text canonical form writes
there. `lumen fmt` is the command that fixes it.

Name resolution comes next. A name with no definition, a module that declares one name twice, and
a binding that hides a name already in scope are each refused, because in Lumen one name has one
definition. `lumen explain` says more about any code that is printed.

Exit codes: 0 when the compiler has nothing to say, 1 when it refuses the program, and 2 when the
file cannot be read.

Report the first thing about a source file that the compiler will not have.

`lumen check` runs the front end over a file and stops at the first refusal. It reads the file,
holds it to canonical form, parses it, resolves every name in it, gives every expression a type,
and checks that every `match` answers for every value it may meet. Nothing is written back.

Loading comes first. `import greeting` names `greeting.lm`, beside the file that writes it, and
every module the file reaches is read before any of them is checked. There is no search path:
a module is the file of that name beside the importing one, or nothing, and an import that names
no such file is refused. `io`, `files`, `list`, `strings`, `map`, and `set` are modules of the
library the compiler carries, so an import of any of them looks for no file at all and a file of
that name beside the importing one does not shadow it.
`map` holds `Map<K, V>`, built by `map.empty` and `map.insert` and read by `map.get`; `set` holds
`Set<T>`, built the same way and read by `set.has_value`. A key is a type the prelude has an `Eq`
instance for, which `Bool`, `Int`, and `String` are. Two modules that import each other are
refused as well, because each would have to be compiled first.

Every module reached is then held to everything below, and a refusal names the file it is in
rather than the file the command named. A module is checked after everything it imports, so a
name reached through an import has the type the other module gave it. What a module offers is
every function it declares and every type it declares, each reached through the module's name. A
generic function is offered like any other: it is written once per set of types it is used at, by
the module that declares it, and a call through an import reaches the method written for the set
that call settled.

Canonical form is the first thing each module is held to: a file that differs is reported with
the line and column it is about, that line under a row of carets, and a `help:` line naming the
text canonical form writes there. `lumen fmt` is the command that fixes it. An import written
after a declaration, or two imports out of sort, is reported here too, and `lumen fmt` does not
fix that one: where an import belongs is said, never rewritten.

How a name is spelled is part of canonical form too, and is reported here for the same reason: a
name written in neither snake_case nor PascalCase, and a declared name of one character, are each
refused with the spelling canonical form gives it or the word it wants. A function whose result
is Bool is held to a name that asks the question it answers, which is reported with the types
because the result read is the one inference settled.

Name resolution comes next. A name with no definition, a module that declares one name twice, and
a binding that hides a name already in scope are each refused, because in Lumen one name has one
definition. A declaration written above something that uses it is refused here as well: a file
reads top down, so the reader meets the intent before the detail.

Type inference comes next. A type written where another is needed, a call with the wrong number
of arguments, a field a record does not declare, and a record built without one of its fields are
each refused. A statement that leaves a value behind and gives it to nothing is refused here as
well, because a dropped `Result` is a swallowed failure; `_ = save(user)` throws a value away on
purpose and says so. Inside a function the types are inferred, so a signature is written where it
documents a boundary rather than on every line.

An `extern` declaration is held here too. It names one member of one Java class and gives it a
Lumen signature: `extern type PrintStream = "java.io.PrintStream"` names the class, and `field`,
`static`, `method`, and `new` name the four kinds of member the JVM has. A parameter or a result
is `Bool`, `Int`, `String`, or a type an `extern type` names, and nothing else crosses; a result
may also be `()`, an `Option` whose `None` is the `null` the member gave back, or a `Result` whose
`Err` holds what a throw said of itself. `Int` compiles to a `long`, and `int` or `char` written after
the kind says the member's own descriptor gives one of those instead, which the call widens to the
`Int` the signature declares: `extern method int length(text: String) -> Int = "length"`. A
parameter writes `int` before its name to say the member takes one there, and the argument is then
narrowed: narrowing loses whatever does not fit, so such a declaration gives back an `Option`, and
an argument outside the `int` range is a `None` that reaches the member not at all. The JVM calls
a method of an interface its own way, and `interface` written after `type` says the class is one:
`extern type interface Path = "java.nio.file.Path"`. A signature naming anything else, a name that
is no Java name, a `derive` of an extern type, a `method` or a `new` whose signature names no
class, a width written where the result is no `Int`, a narrowed parameter whose result is no
`Option`, and a `new` whose result is an interface are each refused. `io`, `files`, and the
readings of a string in `strings` are written over these declarations, so a program reaches the
console, the file system, and the code units of a string without writing one.

`?` is settled here too. It hands the `Err` of a `Result` or the `None` of an `Option` back, and
lands in a function that gives back the same kind, so it never converts one into the other.
A division propagated with `?` in a function that gives back a `Result` is refused for that
reason: a `None` names no error, and a `match` there states which error a zero divisor is. What a
function gives back is still written out, so a body that propagates with `?` ends in `Some(…)` or
`Ok(…)` as any other body does.

How a call passes its arguments is settled here too. A call names all of its arguments or none of
them, written `rename(from: old, to: new)` as a record writes a field, and the names run in the
order the declaration lists the parameters. Where the declaration gives two of its parameters one
type, nothing but the names holds those arguments apart, so the call writes them or is refused:
`rename(old, new)` and `rename(new, old)` both have the types the declaration asks for, and one of
them is wrong. Where the parameter types all differ, either way is allowed and the choice is the
author's. A constructor carries its values in order and has no names, and so does a function the
prelude supplies, so naming the arguments of either is refused rather than read in order.

Exhaustiveness comes last. A `match` that leaves a value of the type it matches unanswered is
refused, and the refusal names a value it does not cover. A `match` that answers for everything
but lists its arms in an order the type does not declare its variants in is refused too, so a new
variant has exactly one place to be handled. `lumen explain` says more about any code that is
printed.

A hole is accepted. `todo("a reason")` stands where a value belongs and takes whatever type is
expected there, so an unfinished body is still resolved, typed, and checked like finished work.
`lumen build` is what refuses a hole, so this is the command to run while one is still there.

`--json` writes the refusal as data rather than as a page to read. One diagnostic is one JSON
object on one line: the file, the code, the message, and the span as byte offsets, with the
advice and the edit where there is either. It goes to standard output, because with the flag the
diagnostic is what was asked for, and standard error stays empty so a run can be piped straight
into a tool. A file the compiler accepts writes nothing at all, and a file that cannot be read at
all is said on standard error and exits 2, with the flag exactly as without it.

Canonical form is the one refusal that carries an edit, because it is the one whose answer the
compiler already knows. That edit is the whole file, and it is exactly the text `lumen fmt`
writes, so a tool applies the compiler's own repair rather than reformatting by hand. Where a
declaration belongs, what a name should have been, and which variant a `match` is missing are all
the author's to decide, so those carry advice and no edit. Applying the edit answers the refusal it
came with and not every refusal the file holds: a file whose imports are also out of order is told
about canonical form first, and checking the repaired file then reports that.

Exit codes: 0 when the compiler has nothing to say, 1 when it refuses the program, and 2 when the
file cannot be read or the directory is no package it can list.

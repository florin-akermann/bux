# Modules and names

Name resolution answers one question for every name in a file: which definition does it mean?
`docs/design.md` section 16 states the rules; this spec states the scopes, the prelude, and the
errors.
The phase consumes the untyped tree from the parser and produces a resolved program, where every
name that has a definition points at it.

## Intent

A reader searching for a name must find its definition and its uses, and nothing else.
That holds only if a name means one thing: no overloading, and nothing hidden behind anything.
The resolver is where the compiler makes that true, before any type is known.

## Modules

One file is one module, and nothing in the file declares it.
Every name the file declares at the top level is public; there is no private declaration.

`import io` brings the module `io` into scope under its own name.
A name inside it is reached as `io.print`, which is a field access on the module and not a name in
the file.
Nothing in the toolchain reads a second file yet, so what a module holds is checked once one can be
loaded.

## Scopes

Names live in two scopes that never mix: a type scope and a value scope.
A type scope is searched where a type is written, and a value scope everywhere else.
`type UserId = UserId(Int64)` therefore declares a type and a constructor without a collision.

A module's type scope holds the prelude's types and every type the file declares.
A module's value scope holds the prelude's constructors, every variant the file declares, every
function it declares, and every module it imports.
Both are collected before any body is walked, so a function may call one declared below it.

A function adds its type parameters to the type scope, and its parameters to the value scope.
A type declaration adds its type parameters to the type scope of its own definition.

Inside a body, a binding is in scope from the statement after it to the end of its block.
`total := total + 1` therefore does not see the `total` it is binding.
A `for … in` binding is in scope in the loop body, and a pattern's bindings in the arm's body.

## The prelude

Every module has these names in scope without importing anything:

```text
types:        Bool  Int  Int64  List  Option  Result  String
constructors: Err  None  Ok  Some
```

They are ordinary declarations of a module the compiler supplies, not keywords.
The prelude becomes Lumen source once a module can be loaded; until then this list is the prelude.

## What is not resolved here

A record field is looked up in the record's type, so `user.name` resolves `user` and leaves `name`.
A field of a record literal names a field of the type being built, and is left the same way.
A name written after `.` is never a name in scope, whether the receiver is a record or a module.

A bare name in a pattern is a use when it names a variant in scope, and a binding otherwise.
That choice is the resolver's, which is why the parser writes both as the same node.

## The errors

| name            | code    | message                              |
|-----------------|---------|--------------------------------------|
| unresolved name | `L0300` | there is nothing named `x`           |
| declared twice  | `L0301` | `x` is declared twice in this module |
| shadowed name   | `L0302` | `x` is already in scope here         |

`L0300` helps with `a name is declared in this file, imported, or supplied by the prelude`.
`L0301` helps with `one name has one definition; rename one of the two`.
`L0302` helps with `rename the inner one; Lumen never hides a name`.

A name written where a type belongs and found only in the value scope is still `L0300`, with a
message saying there is no type of that name.
Two parameters of one function sharing a name are declared twice; a binding that hides a parameter
is shadowed.
A declaration that hides a prelude name is shadowed, because the prelude is already in scope.

Resolution stops at the first error, as parsing does.

## Properties

These hold and are checked with property-based tests:

1. Resolving a program never panics and is deterministic.
2. Every name a resolved program reports a definition for lies within the source.

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
`type UserId = UserId(Int)` therefore declares a type and a constructor without a collision.

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
types:        Bool  Int  List  Option  Result  String
constructors: Err  None  Ok  Some
```

They are ordinary declarations of a module the compiler supplies, not keywords.
The prelude becomes Lumen source once a module can be loaded; until then this list is the prelude.

`Option` declares `Some` and then `None`, and `Result` declares `Ok` and then `Err`.
That order is the one a `match` lists its arms in, which `docs/specs/exhaustiveness.md` requires,
and it is the order `docs/specs/codegen.md` counts a tag in.

## What is not resolved here

A record field is looked up in the record's type, so `user.name` resolves `user` and leaves `name`.
A field of a record literal names a field of the type being built, and is left the same way.
A name written after `.` is never a name in scope, whether the receiver is a record or a module.

A bare name in a pattern is a use when it names a variant in scope, and a binding otherwise.
That choice is the resolver's, which is why the parser writes both as the same node.

## Where a declaration belongs

A file reads top down: the reader meets the intent before the detail.
A declaration is therefore written below what uses it, which `docs/design.md` section 13 requires,
and a declaration written above something that uses it is refused.

The rule is checked here because this is the phase that knows which name means which declaration.
Nothing is moved: the refusal says where the declaration belongs and the author moves it.

Two declarations that use each other are written either way, because no order undoes a cycle.
A use is out of order only when what it uses cannot reach back to it, so recursion is never
reported and neither is a pair that calls each other.

An import is not placed by what uses it.
Canonical form puts every import first and sorted, which settles where it goes without asking;
`docs/specs/formatting.md` states that rule and `L0201` is what refuses it.

## The errors

| name              | code    | message                                         |
|-------------------|---------|-------------------------------------------------|
| unresolved name   | `L0300` | there is nothing named `x`                      |
| declared twice    | `L0301` | `x` is declared twice in this module            |
| shadowed name     | `L0302` | `x` is already in scope here                    |
| written above use | `L0303` | `x` is written above `y`, which uses it         |

`L0300` helps with `a name is declared in this file, imported, or supplied by the prelude`.
`L0301` helps with `one name has one definition; rename one of the two`.
`L0302` helps with `rename the inner one; Lumen never hides a name`.
`L0303` helps with `a file reads top down: move it below what uses it`.

A name written where a type belongs and found only in the value scope is still `L0300`, with a
message saying there is no type of that name.
Two parameters of one function sharing a name are declared twice; a binding that hides a parameter
is shadowed.
A declaration that hides a prelude name is shadowed, because the prelude is already in scope.

Resolution stops at the first error, as parsing does.

## Properties

These hold and are checked with property-based tests:

1. Resolving a program never panics and is deterministic.
2. A module built of pieces that each resolve resolves.
3. A refusal points at a non-empty span that lies within the source.
4. Every name a module declares reports a definition declared at that name.

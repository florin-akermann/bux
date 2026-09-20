# Modules and names

Name resolution answers one question for every name in a file: which definition does it mean?
`docs/design.md` section 16 states the rules; this spec states the scopes, the prelude, and the
errors.
The phase consumes the untyped tree from the parser and produces a resolved program, where every
name that has a definition points at it.
Loading runs before it, and this spec states that too: which file an import names, and in which
order the modules it reaches are compiled.

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

## Loading

`import demo` names the file `demo.lm`, beside the file that imports it.
`io` and `files` are library modules, so an import of either reads the source the compiler
carries and looks for no file at all.
A file beside the importing one is where a module is looked for, and a package is the one other
place: a manifest beside the importing file names the directories a dependency's modules sit in.
`docs/specs/packages.md` states that manifest and the order an import is answered in, and two
files claiming a module of one name are refused there as `L0317`.

Every module a program reaches is loaded before any of them is resolved.
A module is read once however many modules import it, and is the same module to each of them.
That holds because which file a name means is settled before a module already read is looked for,
so two files claiming one name are refused rather than one of them silently standing for both.
Loading walks the imports out from the file the command names, and stops at the first refusal.

A ring of imports is refused.
Two modules that import each other have no order to be typed in, because each needs the other
first.
That is unlike two declarations within one module, which are written either way and resolve either
way.

What a loaded module offers is every function it declares, reached through the module's name.
`demo.helper(2)` is that call, written exactly as `io.print("hi")` is written.

A generic function is offered like any other, and a use of one through an import is a call of the
method the module declaring it writes for the set of types that use settled.
It is written once per set of types it is used at, which `docs/specs/codegen.md` states, and that
page says how a set settled in one module is asked of another.
A function is generic by the type inference settled on it, not by what it wrote: one that writes
no type parameter and leaves its type free is generic in the same way, and is reached the same way.
A constraint such a function writes over one of its type parameters is answered by an instance the
module declaring it reaches, because the body asking is that module's; those are the prelude's,
which `docs/specs/types.md` states as `L0424`.

A module offers the types it declares as well as the functions.
A type is reached through the module's name, as a function is: `demo.User` is the type the module
`demo` declares as `User`.
A variant of it is reached the same way, so a `match` over `demo.Payment` names `demo.Pending`.
A type argument is written after the whole name, so `demo.Held<Int>` applies `Int` to `demo.Held`.

A name reached through a module is never a name in scope, so `demo.User` and a `User` this module
declares are two types and never a clash.
Nothing is brought in by the import but the module's own name.

The type is the one the declaring module declared and not a copy of it: a value built there is the
same value here, and the fields it has and the variants it has are the ones it was declared with.
A record of another module is built by its name, as one of this module is: `demo.User { id: 1 }`.

A value reaches further than a name does.
A function of an imported module gives back what its own module declared, and that may be a type of
a module this one never imported: `relay.got()` gives back a `holder.User` where `relay` imports
`holder` and this module imports only `relay`.
Such a value is held and read as the type it was declared as, fields and variants and all.
What it is never is written: writing `holder.User` needs `holder` in scope, which only an import
puts it in, so a name this module cannot write stays a name it cannot write.

A trait and its instances stay the declaring module's own, exactly as a generic function does.
`demo.User` has no instance of `Eq` here however `demo` came by one, so `==` over two of them is
refused as it is over any type with no instance.
A trait is reached through no module either: `demo.Eq` is not written.
Version 0.1 keeps both where they are declared, and `docs/specs/traits.md` states what an instance
is; what a module offers is the names a reader can write, and an instance has no name to write.

An instance either module declares therefore answers no constraint on an imported generic.
The declaring module's is one this module cannot know it has, because neither offers the other its
instances, and this module's is one that module could not reach.
The prelude's are the only ones left, so a use settling such a type parameter anywhere else is
refused as `L0424` rather than asking for a body the other module could not write.

## Scopes

Names live in two scopes that never mix: a type scope and a value scope.
A type scope is searched where a type is written, and a value scope everywhere else.
`type UserId = UserId(Int)` therefore declares a type and a constructor without a collision.

A module's type scope holds the prelude's types and traits, and every type and trait the file
declares.
A type reached through a module is in neither scope: the module is looked up in the value scope,
and the name after the dot is looked up in what that module declares.
A module's value scope holds the prelude's constructors, every variant the file declares, every
function and trait method it declares, and every module it imports.
An instance declares nothing in either: its methods answer for the name its trait declares, which
`docs/specs/traits.md` states.
A derive declares nothing either, for the same reason: what it writes is an instance.
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
traits:       Add  Div  Eq  Hash  IntegerLiteral  Mul  Neg  Ord  Rem  Show  Sub
constructors: Err  None  Ok  Some
functions:    or  todo
methods:      add  divide  from_literal  hashed  highest  is_equal  is_less  lowest
              multiply  negate  remainder  shown  subtract
```

They are read out of `library/prelude.lm` rather than tabulated in the compiler, which
`docs/specs/library.md` states.

`or(maybe, fallback)` is what an `Option` holds, or the fallback when it holds nothing, and
`docs/specs/arithmetic.md` says why.
`todo(reason)` is a hole, which `docs/specs/holes.md` states.
`Eq` is the trait `==` is, with the instances `docs/specs/traits.md` names.
`Hash` and `Show` are what a value opts into a hash and into text with, which the same spec
states.
`IntegerLiteral` is the trait a whole-number literal is, which `docs/specs/literals.md` states.
The other seven traits are the ones the other operators are, which `docs/specs/operators.md`
names along with the instances the compiler supplies for each.

They are ordinary declarations of a module the compiler carries, not keywords.
An import never reaches the prelude: a prelude name is written bare, and an import brings a module
into scope under its name rather than the names inside it.
Another module of the library is imported as any module is, which `docs/specs/library.md` states.

`Option` declares `Some` and then `None`, and `Result` declares `Ok` and then `Err`.
That order is the one a `match` lists its arms in, which `docs/specs/exhaustiveness.md` requires,
and it is the order `docs/specs/codegen.md` counts a tag in.

## What is not resolved here

A record field is looked up in the record's type, so `user.name` resolves `user` and leaves `name`.
A field of a record literal names a field of the type being built, and is left the same way.
A name reached through a module is never a name in scope either.
What such a name means is inference's to say, because the module declaring it is what says so.
A type and a pattern reached through a module are resolved the same way: the module is a name in
scope and what follows the dot is not.

The name of a call is the exception, and only where a module is not what is before the dot.
`maybe.or(0)` is the call `or(maybe, 0)`, which `docs/specs/calls.md` states, so `or` is a name of
this module and is resolved here like any other.
The name before the dot is what says which of the two a call is, and nothing else is.

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
| name that is no value | `L0304` | `x` is a function, so it is written as a call |
| assigned but no `var` | `L0305` | `x` is not a `var`, so it is never assigned to |
| module with no file | `L0306` | there is no module named `demo`                   |
| ring of imports   | `L0307` | `demo` imports `main`, which imports `demo`       |
| module is two files | `L0317` | `demo` is both `../shapes/demo.lm` and `demo.lm` |
| reached through no module | `L0313` | `user` is a module in neither scope, and a type is reached through one |

`L0300` helps with `a name is declared in this file, imported, or supplied by the prelude`.
`L0301` helps with `one name has one definition; rename one of the two`.
`L0302` helps with `rename the inner one; Lumen never hides a name`.
`L0303` helps with `a file reads top down: move it below what uses it`.
`L0304` helps with `version 0.1 reaches a function by calling it; write the call`.
`L0304` helps a module with ``a module is what a name is reached through, as `io.println` is``.
`L0305` helps with ``mutation is explicit: bind it with `var`, or bind a new name``.
`L0306` helps with ``a module is a file beside this one, or one of a package it depends on``.
`L0307` helps with `a module is compiled after what it imports, and a ring has no such order`.
`L0317` helps with `one name has one definition; rename one of the two modules`.
`L0313` helps with ``a type of another module is reached through the import: write `demo.User```.

`L0313` is the name on the left of the dot of a type or of a pattern, which is a module or
nothing at all.
A module the file does not import is `L0300`, because the name is looked up in the value scope
before it is asked to be a module.
A name that module does not declare is `L0414`, which is raised where every other name reached
inside a module is; `docs/specs/types.md` states it.

`L0306`, `L0307`, and `L0317` are raised while loading, before any module is resolved.
Each points at the import that was being followed, in the file that wrote it.
A manifest the loader could not read is refused there too, which `docs/specs/packages.md` states
as `L0315` and `L0316`.

A name written where a type belongs and found only in the value scope is still `L0300`, with a
message saying there is no type of that name.
Two parameters of one function sharing a name are declared twice; a binding that hides a parameter
is shadowed.
A declaration that hides a prelude name is shadowed, because the prelude is already in scope.

Version 0.1 reaches a function by calling it, which `docs/design.md` section 11 states.
A function name is `L0304` wherever it is written but as the name of a call.
`held := helper`, `helper = 1`, and `filter(users, is_active)` are each refused at the name.
A module name is `L0304` wherever it is written but on the left of a `.`.
Its message is `x` is a module, so a name inside it is what is written.
Neither has a type or a shape in version 0.1, so code generation is never handed one.

A name reached inside a module is `L0304` too, and inference is what raises it: only inference
knows whether that module declares a function of that name or a variant carrying nothing.
`demo.helper` outside a call is refused; `demo.Pending` is a value and is written as it stands.
`docs/specs/types.md` states the code among the ones inference raises.

The name an assignment names is a `var` binding, which `docs/design.md` section 10 states.
Anything else is `L0305`, pointing at the name on the left of the `=` or the `+=`.
A `:=` binding, a parameter, and a `for … in` binding each never change.
Neither does a name a pattern binds, which an arm's body has no statement position to assign in.
A constructor never changes either: it names a way to build a value, not a place to put one.
Without the refusal it lowers into a store with nowhere to write, and does nothing at all.

Resolution stops at the first error, as parsing does.

## Properties

These hold and are checked with property-based tests:

1. Resolving a program never panics and is deterministic.
2. A module built of pieces that each resolve resolves.
3. A refusal points at a non-empty span that lies within the source.
4. Every name a module declares reports a definition declared at that name.

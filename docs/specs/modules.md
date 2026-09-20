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
There is no search path: a module is the file of that name beside the importing one, or nothing.
`io` and `files` are supplied by the compiler, so an import of either looks for no file at all.

Every module a program reaches is loaded before any of them is resolved.
A module is read once however many modules import it, and is the same module to each of them.
Loading walks the imports out from the file the command names, and stops at the first refusal.

A ring of imports is refused.
Two modules that import each other have no order to be typed in, because each needs the other
first.
That is unlike two declarations within one module, which are written either way and resolve either
way.

What a loaded module offers is every function it declares, reached through the module's name.
`demo.helper(2)` is that call, written exactly as `io.print("hi")` is written.

A generic function stays that module's own too.
It is written once per set of types it is used at, which `docs/specs/codegen.md` states, and the
module declaring it writes only the sets its own body reaches.
A function is generic by the type inference settled on it, not by what it wrote: one that writes
no type parameter and leaves its type free is generic in the same way.
`docs/specs/types.md` states that as `L0417`.

A type a module declares stays that module's own in version 0.1.
A type is written as a bare name, and no name reaches into a module, so an importing module has no
way to write one.
A function whose signature names one is therefore refused where it is reached rather than where it
is declared, which `docs/specs/types.md` states as `L0416`.
A module builds what it likes and offers what the modules importing it can name.

## Scopes

Names live in two scopes that never mix: a type scope and a value scope.
A type scope is searched where a type is written, and a value scope everywhere else.
`type UserId = UserId(Int)` therefore declares a type and a constructor without a collision.

A module's type scope holds the prelude's types and traits, and every type and trait the file
declares.
A module's value scope holds the prelude's constructors, every variant the file declares, every
function and trait method it declares, and every module it imports.
An instance declares nothing in either: its methods answer for the name its trait declares, which
`docs/specs/traits.md` states.
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
traits:       Add  Div  Eq  IntegerLiteral  Mul  Neg  Ord  Rem  Sub
constructors: Err  None  Ok  Some
functions:    or  todo
methods:      add  divide  from_literal  highest  is_equal  is_less  lowest  multiply
              negate  remainder  subtract
```

`or(maybe, fallback)` is what an `Option` holds, or the fallback when it holds nothing, and
`docs/specs/arithmetic.md` says why.
`todo(reason)` is a hole, which `docs/specs/holes.md` states.
`Eq` is the trait `==` is, with the instances `docs/specs/traits.md` names.
`IntegerLiteral` is the trait a whole-number literal is, which `docs/specs/literals.md` states.
The other seven traits are the ones the other operators are, which `docs/specs/operators.md`
names along with the instances the compiler supplies for each.

They are ordinary declarations of a module the compiler supplies, not keywords.
Loading does not reach the prelude: a prelude name is written bare, and an import brings a module
into scope under its name rather than the names inside it.

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
| name that is no value | `L0304` | `x` is a function, so it is written as a call |
| assigned but no `var` | `L0305` | `x` is not a `var`, so it is never assigned to |
| module with no file | `L0306` | there is no module named `demo`                   |
| ring of imports   | `L0307` | `demo` imports `main`, which imports `demo`       |

`L0300` helps with `a name is declared in this file, imported, or supplied by the prelude`.
`L0301` helps with `one name has one definition; rename one of the two`.
`L0302` helps with `rename the inner one; Lumen never hides a name`.
`L0303` helps with `a file reads top down: move it below what uses it`.
`L0304` helps with `version 0.1 reaches a function by calling it; write the call`.
`L0304` helps a module with ``a module is what a name is reached through, as `io.println` is``.
`L0305` helps with ``mutation is explicit: bind it with `var`, or bind a new name``.
`L0306` helps with ``a module is a file beside this one: write `demo.lm```.
`L0307` helps with `a module is compiled after what it imports, and a ring has no such order`.

`L0306` and `L0307` are raised while loading, before any module is resolved.
Both point at the import that was being followed, in the file that wrote it.

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

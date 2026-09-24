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
Every name the file declares at the top level is public, unless its declaration is `private`.

## A private declaration

`private` is a keyword, and it comes before `fn`, `type`, `trait`, or `process`.
Those four declare a name that another module can reach, and nothing else does.
`private instance`, `private derive`, `private test`, `private import`, and `private extern` are
each `L0100`, at the word after `private`.
Canonical form writes `private`, one space, and then the keyword, as `private fn opening()`.

A private name is reached only in its module, and every rule of this spec holds for it there.
An import that reaches it is refused as `L0322`, at the name after the dot.
That holds for a function, a process after `spawn`, and a type written through the module.
A variant of a private type is private with the type, so `kept.Low` is refused too.
A private record type is built through its name, so that name is refused as a constructor too.
A type and a constructor are in two namespaces, and `L0322` keeps them apart.
So a private constructor `Box` does not keep a public type `Box`, as `namesakes_reached.bx` shows.
A private trait needs no refusal of its own, because a module offers no trait.

A value reaches further than a name does, as the section on loading states.
A public function can give back a value of a private type, and a module that imports it holds that
value and passes it on.
It never writes the name of that type, and it never builds or matches a variant of it.
The instances of a private type travel with its values, as the instances of any type do.

A private name of the prelude is reached only in the prelude, so no other module has it in scope.

`import io` brings the module `io` into scope under its own name.
A name inside it is reached as `io.print`, which is a field access on the module and not a name in
the file.

## Loading

`import demo` names the file `demo.bx`, beside the file that imports it.
A source file is named `<module>.bx`, and the loader reads no file of another name.
A command given a file whose name does not end in `.bx` refuses it as `L0605`, before it compiles.
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

A module that imports itself is the ring of one module, and it is refused as `L0307` too.
The message names the module once, as in `` `demo` imports itself ``.
A library module is held to the same rule, so `import list` in `library/list.bx` is refused.
A ring is refused at the name in the import that closes it, before the loader follows the import.

What a loaded module offers is every function it declares that is not `private`, reached through
the module's name.
`demo.helper(2)` is that call, written exactly as `io.print("hi")` is written.
A module offers each process it declares in the same way, and `spawn demo.Ticker(5)` starts one.
`docs/specs/concurrency.md` states how a process is reached.

A generic function is offered like any other, and a use of one through an import is a call of the
method the module declaring it writes for the set of types that use settled.
It is written once per set of types it is used at, which `docs/specs/codegen.md` states, and that
page says how a set settled in one module is asked of another.
A function is generic by the type inference settled on it, not by what it wrote: one that writes
no type parameter and leaves its type free is generic in the same way, and is reached the same way.
A constraint such a function writes over one of its type parameters is answered by the instance
the type that use settled it on has, wherever that type is declared.
The module writing the use is the one that proves the instance is there, and the method written
for the set calls it by name, which `docs/specs/codegen.md` states.

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

A trait stays the declaring module's own: `demo.Eq` is not written.
What a module offers is the names a reader can write, and a trait is not among them.

The instances of a type travel with the type, which `docs/design.md` section 8 settles.
An instance belongs in the module that declares its trait or in the module that declares its type.
One trait and one type have one instance in a program, so a type has the same instances everywhere.
A module that reaches `demo.User` reaches every instance `demo` gives `User`, and the prelude's.
An instance that `demo` derives travels in the same way as one that `demo` writes.
`==` over two `demo.User`s is accepted wherever `demo` gives `User` an instance of `Eq`.
The instance is a method of the class of `demo`, and `docs/specs/codegen.md` states the call.

An instance has no name to write, so it adds no name to what a module offers.
It comes with the type it is for, and a value of that type that reaches a module brings it along.
An instance of a trait that `demo` keeps to itself stays in `demo`.
Nothing outside `demo` can write the name of that trait, so nothing outside it can ask for one.
An instance whose constraint names such a trait stays in `demo` too.
No other module can answer that constraint, so outside `demo` the type has no such instance.

A constraint on an imported generic is answered by the instance of the type that the use settled.
That type can be declared in any module of the program, and the instance travels with it.
A use that settles the parameter on `demo.User` is `L0418` only where no module has the instance.
The trait is what has to be named twice over, once in the constraint and once in the instance, so
a constraint over a trait the declaring module keeps to itself is refused as `L0424`.

## The loader written in Bux

`compiler/modules.bx` is this loader, written in Bux.
`modules.load(path)` reads the module at `path` and every module it reaches, as this section and
`docs/specs/packages.md` state.
It gives the modules in order, dependencies before dependents.
Each module keeps its name, the path it was read from, its source, and the tree of
`compiler/parser.bx`.

It stops at the first refusal, which has a file, a code, a span, a message, and a help line.
A span counts UTF-8 bytes, as a span of `compiler/lexer.bx` does.
A library module is a resource on the class path, which `docs/specs/library.md` states.
Whether two routes reach one file is the JVM's canonical path of each.

A file that the loader cannot read is refused with the path that the import named.
The loader asks whether a file is there before it looks beside a module or in a package.
It asks through `java.io.File.isFile`, so a directory of that name is no module.

`modules.printed(path)` gives the answer in the form that `tests/loading.bx` compares.
A load is `loaded`, and then one line for each module: its name and its path.
A refusal is `refused` and the path of the file it is about, then the code, span, and message,
then `help:` and the help.
A file that cannot be read is `unreadable` and its path.

`tests/loading.bx` holds the loader to its properties, on trees of modules and manifests it draws.

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
`let total = total + 1` therefore does not see the `total` it is binding.
A `for … in` binding is in scope in the loop body, and a pattern's bindings in the arm's body.

## The prelude

Every module has these names in scope without importing anything:

```text
types:        Bool  Int  List  Next  Option  Process  Result  Sent  String  Waiting
traits:       Add  Div  Eq  Hash  IntegerLiteral  Mul  Neg  Ord  Rem  Show  Sub
constructors: Continue  Delivered  Done  Err  MailboxFull  Milliseconds  NoLimit  NoWait
              None  Ok  ProcessEnded  Some
functions:    ended  or  send  todo
methods:      add  divide  from_literal  hashed  highest  is_equal  is_less  lowest
              multiply  negate  remainder  shown  subtract
```

They are read out of `library/prelude.bx` rather than tabulated in the compiler, which
`docs/specs/library.md` states.

`or(maybe, fallback)` is what an `Option` holds, or the fallback when it holds nothing, and
`docs/specs/arithmetic.md` says why.
`todo(reason)` is a hole, which `docs/specs/holes.md` states.
`Eq` is the trait `==` is, with the instances `docs/specs/traits.md` names.
`Hash` and `Show` are what a value opts into a hash and into text with, which the same spec
states.
`IntegerLiteral` is the trait a whole-number literal is, which `docs/specs/literals.md` states.
The other seven traits are the ones the other operators are, which `docs/specs/operators.md`
names along with the instances the library writes for each.

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

The name after a dot is never resolved here, and the name of a call is no exception.
A call through a module is inference's to say, as every other name reached through one is.
A call with a value before the dot is refused by inference as `L0433`, which
`docs/specs/arguments.md` states, so `maybe.or(0)` is written `or(maybe, 0)`.

A bare name in a pattern is a use when it names a variant in scope, and a binding otherwise.
That choice is the resolver's, which is why the parser writes both as the same node.

## Where a declaration belongs

A file reads top down: the reader meets the intent before the detail.
A declaration is therefore written below what uses it, which `docs/design.md` section 13 requires,
and a declaration written above something that uses it is refused.

The rule is checked here because this is the phase that knows which name means which declaration.
The refusal says where the declaration belongs, and `bux fmt` moves it there.
`resolver.top_down` gives `bux fmt` that order, which `docs/specs/formatting.md` states.

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
| module is two files | `L0317` | `demo` is both `../shapes/demo.bx` and `demo.bx` |
| reached through no module | `L0313` | `user` is a module in neither scope, and a type is reached through one |
| import read by nothing | `L0320` | the import `strings` is read by nothing |
| private name reached | `L0322` | `opening` is private to `kept` |

`L0300` helps with `a name is declared in this file, imported, or supplied by the prelude`.
`L0301` helps with `one name has one definition; rename one of the two`.
`L0302` helps with `rename the inner one; Bux never hides a name`.
`L0303` helps with ``a file reads top down: `bux fmt` moves it below what uses it``.
`L0304` helps with `version 0.1 reaches a function by calling it; write the call`.
`L0304` helps a module with ``a module is what a name is reached through, as `io.println` is``.
`L0305` helps with ``mutation is explicit: bind it with `var`, or bind a new name``.
`L0306` helps with ``a module is a file beside this one, or one of a package it depends on``.
`L0307` helps with `a module is compiled after what it imports, and a ring has no such order`.
`L0317` helps with `one name has one definition; rename one of the two modules`.
`L0313` helps with ``a type of another module is reached through the import: write `demo.User```.
`L0320` helps with `read it, or remove it`.
`L0322` helps with ``a private name is reached only in `kept`, the module that declares it``.

`L0320` is the import whose module name nothing writes before a dot, at the name in the import.
A run compiles an example in the module scope, so a module name before a dot there is a read.
`docs/specs/doc-examples.md` states that an example is a claim in Bux, and a run compiles it.
`docs/specs/unused.md` states the same rule for a binding and for a parameter, which is `L0321`.

`L0322` is raised where a name reached inside a module is looked up, which is inference.
The resolver reads one module and knows nothing another module declares, so it cannot raise it.
It is raised before `L0414`, so a private name is never reported as a name its module lacks.

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
`let held = helper`, `helper = 1`, and `filter(users, is_active)` are each refused at the name.
A module name is `L0304` wherever it is written but on the left of a `.`.
Its message is `x` is a module, so a name inside it is what is written.
Neither has a type or a shape in version 0.1, so code generation is never handed one.

A name reached inside a module is `L0304` too, and inference is what raises it: only inference
knows whether that module declares a function of that name or a variant carrying nothing.
`demo.helper` outside a call is refused; `demo.Pending` is a value and is written as it stands.
`docs/specs/types.md` states the code among the ones inference raises.

The name an assignment names is a `var` binding, which `docs/design.md` section 10 states.
Anything else is `L0305`, pointing at the name on the left of the `=` or the `+=`.
A `let` binding, a parameter, and a `for … in` binding each never change.
Neither does a name a pattern binds, which an arm's body has no statement position to assign in.
A constructor never changes either: it names a way to build a value, not a place to put one.
Without the refusal it lowers into a store with nowhere to write, and does nothing at all.

Resolution stops at the first error, as parsing does.

## The resolver written in Bux

`compiler/resolver.bx` is this resolver, written in Bux.
It reads the name and the tree of each module that `compiler/modules.bx` loads.
The tree is the tree of `compiler/parser.bx`, and nothing in it changes.
The answer is the tree and, for each name that has a definition, the definition that it means.

`resolver.prelude_of(program)` reads the names of the prelude out of its parsed tree.
The resolver reads no file and no resource, so the caller gives it the prelude.
`resolver.resolve(program, module, prelude)` resolves one module against that prelude.
`resolver.prelude_resolved(program)` resolves the prelude itself, with the names the JVM holds.
`resolver.top_down(program, prelude)` gives the program with each declaration below what uses it.
A declaration moves only where a use makes it move.
Two declarations that use each other keep the order the author wrote.

It stops at the first refusal, which has a code, a span, a message, and a help line.

`resolver.printed(source, prelude)` gives the answer in the form that `tests/naming.bx` compares.
A resolved module is `resolved`, and then one line for each name that has a definition.
A line is the span of the name, `type` or `value`, the kind of definition, and its origin.
The kinds are `module`, `type`, `trait`, `type-parameter`, `constructor`, `function`,
`trait-method`, `parameter`, `local`, and `variable`.
The origin is `prelude`, or `declared` and the span of the name that declares it.
The lines are in the order of their spans, and a `type` line comes before a `value` line.
A refusal is `refused`, then the code, span, and message, then `help:` and the help.
A source that does not parse is `unparsed`.

`tests/naming.bx` holds the resolver to the properties below, on modules drawn at random.

## Properties

These hold and are checked by drawn properties, each a test of `tests/`:

1. Resolving a program never panics and is deterministic.
2. A module built of pieces that each resolve resolves.
3. A refusal points at a non-empty span that lies within the source.
4. Every name a module declares reports a definition declared at that name.

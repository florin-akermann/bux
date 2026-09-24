# Type inference

## Intent

Every expression of a module gets a type, and nothing in the module is written down twice to say so.
A signature is written where it documents a boundary; inside a function, inference does the work.
Every function a program writes is such a boundary, so it writes its whole signature.
The phase consumes the resolved tree and yields the same tree with a type against every expression.

A type is never guessed from a runtime representation.
`type UserId = UserId(Int)` makes a type that is not `Int`, however the JVM ends up holding it.

## The types

The types of version 0.1 are:

```text
Int  String  Bool  ()
List<T>  Option<T>  Result<T, E>
a declared record or ADT, with its type arguments
a function, written (Int, String) -> Bool
```

`Int` is a whole number, 64 bits wide, and the only one the prelude supplies.
`()` is the type of a function that returns nothing interesting.
`List<T>` is written `[first, second]`, and `for` is what walks one.
`Option` and `Result` are ordinary algebraic data types supplied by the prelude.

A type the author writes is the type they get.
`fn wrap(raw: Int) -> UserId` is refused if its body returns an `Int`.
A written type carries one argument for each its declaration lists, so `List<Int, Int>` is refused.

A declared type does not hold a value of itself.
`type Node = { number: Int, next: Node }` is refused where it is declared, because a value has no
null: that field would hold a whole `Node`, whose own field would hold another, without end.
A ring of declarations is refused the same way, so an `A` holding a `B` that holds an `A` is too.
The refusal names the ring in the order it runs, and points at the declaration it came back to.
`Option<Node>` is how a type holds another of its own kind, and it is accepted.

Only a record's own fields are followed, because a record is what holds another by value.
A field written as the base of an ADT holds whichever variant it was handed, which is a reference.
A type written as an argument is not followed either, for the same reason: it is carried as one.

An imported module is a type of its own that has nothing inside it.
Version 0.1 brings a module into scope and has no way yet to reach a name in one.

## Every function writes its signature

Every function a program writes states the type of each parameter and the type of its result.
That holds for a function of a module, a method of an `instance`, and `start` and `receive`.
A method of a `trait` writes its result too, and `L0436` refuses it where the trait is declared.
A function that gives back `()` writes `-> ()`, so there is one way to write a signature.
A `let`, a `var`, a `for` binding, and a pattern write no type, because inference works in a body.
A test is no function, and it writes no signature.

A type left out of a signature is `L0436`.
The check runs after inference of the whole module, so the help spells the settled signature.
For `fn same<T>(value) -> T`, the help is `` write the signature `fn same<T>(value: T) -> T` ``.
A type that nothing settled is `_` there, as a diagnostic writes it.
The span is the parameter that leaves its type out, or the name where the result is left out.
The functions are read in the order the module writes them, and the first omission is refused.
A function of a `process` that leaves a type out is `L0806` first, as `concurrency.md` states.
`bux fmt` writes no type in, because a signature is the author's to write.

## Inference

A function is inferred from its signature inward.
Inference runs before the signature check, so a type left out gets a fresh type variable.
The body is inferred and its type is unified with the result type.

Unification is the usual one: two types match when they are the same shape over matching parts.
A variable stands for whatever it is unified with, and a variable never stands for a type that
contains it.

Literals type as themselves: a whole number is `Int`, a quoted string is `String`, `true` is `Bool`,
and `()` is `()`.

A written list is a `List<T>` over the one type its elements share, so `[1, 2]` is a `List<Int>`.
Each element is unified with the ones before it, and one that does not fit is `L0400` where it is
written.
`[]` writes a list of nothing, and takes the element type from whatever it is unified with; where
nothing unifies with it, that type is left unsettled, and nothing downstream needs it.

Writing a list builds one, and writing it inside a loop builds one each turn.
Every element is evaluated once, left to right, in the order it is written.
Version 0.1 has no way to add to a list it has already built, so a list is written whole.

Every operator is a trait method, which `docs/design.md` section 8 states.
`library/prelude.bx` writes the instances of each over the types the JVM holds, which is what a
program gets without writing one of its own:

```text
+                        Int + Int, or String + String
- * / %                  Int
< <= > >=                Int
== !=                    two values of one type that has `Eq`
&& ||  and prefix !      Bool
prefix -                 Int
```

An addition whose type is still unknown when its function has been inferred is an addition of
`Int`s.
This is the one default in the language, and it is here because version 0.1 has no typeclasses.
Once a literal takes the type its context expects, the same default settles a literal instead.
A comparison whose type is still unknown at that point is a comparison of `Int`s for the same
reason.

`==` and `!=` compare two values of a type that has `Eq`.
Version 0.1 has no `derive`, so the types that have `Eq` are the three the library ships: `Int`,
`Bool`, and `String`.
Comparing two values of any other type is refused, and stays refused until version 0.2 lets a
type derive `Eq`.
The check runs once the function the comparison is written in has been inferred, so the type it
names is the settled one.

A block's type is the type of its last statement when that statement is an expression, and `()`
otherwise.
Every statement above the last one is written for its effect, so its type is `()` and anything
else is `L0408`; `docs/specs/discarding.md` states which blocks give their last statement's value
away and which discard that one too.
A block whose last statement is `return`, `break`, or `continue` has whatever type is asked of it,
because control has already left.

`if` without an `else` has type `()`, and every branch of it must have type `()`.
`if` with an `else` has the one type all of its branches share.
Every arm of a `match` has that one type too, and every pattern matches the scrutinee's type.

`for x in xs` asks that `xs` is a `List<T>`, and binds `x` to `T`.
`for <condition>` asks that the condition is a `Bool`.
A loop has no value, so whatever its body leaves behind is discarded.

`e?` asks that `e` is a `Result<T, E>`.
It has type `T`, and the enclosing function returns `Result<_, E>` for that same `E`.

A field is reached through `.`, so a field access waits until the type of what it is reached
through is known.
Each one is settled when the function it is written in has been inferred.
They settle before the body of that function meets the result it declares, so a field is reported
where it is written.

`User { id: 1 }` builds a record and gives a value to every field the record declares.
`user { active: false }` updates one and names only the fields that change, so it has the type it
already had.
Either way a field is given a value once, because nothing says which of two values would win.
Which of the two a name followed by braces is depends on what the name means, which name
resolution has already decided.

A pattern that names fields binds the ones it names and leaves the rest alone.
`Authorized { authorization_id }` binds `authorization_id` at the type the variant declares it.

## Generalisation

A top-level function is polymorphic over the type parameters it declares.
`fn identity<T>(value: T) -> T` is `∀T. (T) -> T`, and a `T` inside its body is a type of its own
that unifies with nothing else.
That is what makes `fn identity<T>(value: T) -> T { 1 }` a mismatch rather than a proof.

A function is then generalised over the variables that its body left free.
A function that writes its whole signature leaves no variable free but its type parameters.
Functions are inferred bottom up, which is the order a module is read in reverse.
A declaration sits below what uses it, which `docs/specs/modules.md` requires, so walking upwards
reaches a function before anything that calls it.

`let total = count(users)` generalises what it binds, so a name bound this way is as polymorphic as
the value it was given.
`var total = 0` does not, because a mutable binding is assigned to later and must stay one type.

## An `Option` never carries `()`

`Some(())` says only that a value is there, and `None` says that it is not.
That is `Bool` written a second way, and at the boundary to the platform it is a flag for `null`.
`docs/rationale.md` section 5 gives the reason, and the compiler refuses `Option<()>` with `L0432`.

A written `Option<()>` is refused where it is written: in a signature, a field, or an `extern`.
The span is the written type, and in `List<Option<()>>` it is the inner `Option<()>`.
The declarations are read before any body, so a written one comes before every inferred one.
The refusal comes before the `extern` boundary check, so an `extern` that writes it gets `L0432`.

An inferred `Option<()>` is refused at the expression whose type became one or holds one.
The phase reads the types after inference has settled every function of the module.
It reads the functions in the order the module writes them, and the expressions in the same order.
It reads the expressions inside an expression first, so the refusal names the innermost one.
The type of a function does not count, because a call of the function gives the type that holds it.
These four are refused, each at the span that `tests/spec/unit` states for it:

```text
fn held() -> Option<()>      // the written type Option<()>
match Some(()) { … }         // the expression Some(())
match None { Some(inside) => is_given(inside) … }   // the None, where is_given takes ()
match first(all) { … }       // first<T> over a List<()>, at the call first(all)
```

`Result<(), E>` is accepted, because its `Err` carries a reason the caller could not work out.
A `Bool` is the answer for a flag, and a `Result<(), E>` for a failure with a reason.
A declared type with a field of type `Option<T>` is accepted at `T = ()`.
An expression whose type is that field's `Option<()>` is then refused, as every other one is.

## The errors

| name              | code    | message                                          |
|-------------------|---------|--------------------------------------------------|
| type mismatch     | `L0400` | expected `Int`, found `String`                   |
| wrong arity       | `L0401` | `describe` takes 1 argument but 2 were given     |
| unknown field     | `L0402` | `User` has no field named `total`                |
| infinite type     | `L0403` | this would have a type that contains itself      |
| missing field     | `L0404` | `User` needs a field named `id`                  |
| field given twice | `L0405` | `User` is given `id` twice                       |
| no operator       | `L0406` | `User` has no `Eq`, so `==` is not written over it |
| zero divisor      | `L0407` | this divisor is zero, so there is no answer      |
| value discarded   | `L0408` | `Result<(), Error>` is left here and nothing takes it |
| unnamed arguments | `L0409` | `rename` gives two parameters the type `String`, so this call names its arguments |
| misnamed argument | `L0410` | this argument is named `to`, and the parameter here is `from` |
| no names to write | `L0411` | `Span` is a constructor, so it carries its values in order and names none |
| flag parameter    | `L0412` | this parameter is a `Bool`, so a call of `open` passes `true` and says no more |
| not a predicate   | `L0413` | `active` gives back a `Bool`, so its name asks the question it answers |
| holds itself      | `L0415` | `Node` holds `Node`                              |
| name inside a module that is no value | `L0304` | `holding.held` is a function, so it is written as a call |
| literal misfit    | `L0420` | `5000000000` does not fit `Int32`, which holds `-2147483648` to `2147483647` |
| bound is not a number | `L0421` | `lowest` of `Int32` is read rather than run, so it is one whole number |
| trait stays in its module | `L0424` | `holder.labelled` requires `Named`, which `holder` declares and nothing here names |
| unit in an option | `L0432` | `Option` never carries `()`, because `Some(())` says no more than `true` |
| call on a value   | `L0433` | a dot after a value reads a field, so this call of `or` is written plainly |
| signature left out | `L0436` | `count` of `doubled` states no type, and a function writes its whole signature |

`L0406` covers every operator, because every operator is a trait method and a type is written
with one exactly where it has that trait's instance, which `docs/specs/operators.md` states.
`L0420` and `L0421` are what `docs/specs/literals.md` states: a whole number is held to the range
the instance of the type it settled on states, and that instance states it as two whole numbers.
A whole number written at a type that takes none is `L0400`, because a whole number nothing else
settles is an `Int` and an `Int` is not that type.
`L0402` also says the type reached through `.` is not known, when inference never settled it.
A variant that carries its values in order has no field to write against, so that is `L0402` too.
A name reached inside a module is never `L0402`: every module in scope is one loading read, so
what it declares is what answers, and a name it does not declare is `L0414`.
A generic function a module declares is offered like every other, which `docs/specs/modules.md`
states, so a call of one through an import is typed against the scheme that module wrote.
A constraint the declaration wrote over a type parameter is answered here, by the instance the
type this use settles it on has, and `docs/specs/codegen.md` states how the method written for
that set of types calls that instance by name.
`L0424` is the one thing such a call is held to beyond what a call of any other is: a trait stays
where it is declared, which `docs/specs/modules.md` states, so a constraint over a trait the
declaring module keeps to itself is answered by nothing here and is refused rather than compiled.
The prelude's traits are the ones two modules both name, and a constraint over one of those is
answered as every other constraint is, or refused as `L0418`.
A generic whose type parameters carry no constraint is reached at any type at all.
A type that module declares is offered, and a signature naming one is reached like any other.
`L0414` is also a type reached through a module that the module does not declare, because a type
is a name reached inside a module as a function is.
`L0304` is the resolver's code, raised here because only inference knows what a module offers:
a name reached inside one is a function or a value, and version 0.1 holds no function.
A variant that module declares which carries nothing is a value, so it is written as the name
alone; every other name inside a module is written as the call `docs/specs/modules.md` states.
`L0401` counts the arguments of a written type as well as those of a call.
`L0407` is raised where a division is written with a `0` the compiler can already see.
`L0409`, `L0410`, and `L0411` are about how a call passes its arguments, which
`docs/specs/arguments.md` states.
All three are reached after `L0401` and after the arguments have met the parameter types, because
how many there are and what they are is each settled before which of them is which.
`L0411` also says a name was written on a call of the prelude, which declares none to check.
`L0433` is a call with a value in front of the name, which `docs/specs/arguments.md` states.
It is raised before anything else about the call, and its `help:` is the plain call.
`L0412` is a parameter that is a bare `Bool`, which `docs/specs/arguments.md` states as well.
It is raised where the parameter is written, because the declaration is what changes.
It is reached after the body, because the type it reads is the one inference settled.
An instance method is reached through its trait rather than at the instance: the trait wrote the
signature, so a trait's own method is held to the rule and an instance's is not, which
`docs/specs/arguments.md` states.
`L0413` is a function whose result is `Bool` and whose name asks nothing, which
`docs/specs/naming.md` states.
It reads the same result and is reached at the same point, after `L0412` and for the same reason.
A function whose parameters and result are all `Bool` is about `Bool`, and is the one carve-out.
`L0432` helps with ``use `Bool` for a flag, and `Result<(), E>` for a failure with a reason``.

`/` and `%` give back `Option<Int>` rather than `Int`, which `docs/specs/arithmetic.md` states.
`L0400` is what an `Option<Int>` met where an `Int` belongs is refused with, as anything else is.

Inference stops at the first error it reaches, which is the one lowest in the file.

## The type inference written in Bux

`compiler/types.bx` is this phase, written in Bux.
It reads each module that `compiler/resolver.bx` resolves, in the order `compiler/modules.bx` loads.
Seven more modules hold the parts of the phase, one concern each, and no two import each other.
`compiler/unify.bx` holds the types, the unification table, and the schemes.
`compiler/refusal.bx` holds each refusal, with its code, its message, and its help.
`compiler/boundary.bx` holds the rules for a type that crosses to Java.
It asks `java.lang.Character` whether each code point of a Java name is a letter or a number.
`compiler/surface.bx` holds what a module offers the modules that import it.
`compiler/declared.bx` holds what a module declares, with the checks of each declaration.
`compiler/infer.bx` walks each function and settles what the walk left open.
`compiler/carried.bx` reads the settled types, and refuses an `Option<()>` that inference reached.
`compiler/types.bx` refuses a signature that leaves a type out, after the walk of the module.

The unification table is a value: each step gives back a new table, and no step changes one.
The table is a `Map` from each type variable to the type it was settled on.
Each other table of the phase is a `Map` or a `Set` of the library, or a `List`.
A name is kept under its key, which is where it is declared, its prelude name, or `module.Name`.

`types.prelude_read(source)` reads the declarations of the prelude out of its source.
The phase reads no file for the prelude, so the caller gives it the prelude.
`types.check(resolved, imported, prelude)` infers one module against the surfaces of its imports.
`types.surface_of` gives what an inferred module offers, and `surface.offering` adds it.

It stops at the first refusal, which has a code, a span, a message, and a help line.
A module that the phase infers has the `bux api` page of `docs/specs/api-surface.md`.

`tests/typing.bx`, `tests/conventions.bx`, and `tests/signed.bx` hold the phase to its properties.
`tests/siblings.bx` holds the page of each example to its `.api` file.

## Properties

These hold and are checked by drawn properties in the runner:

1. Inferring a program never panics and is deterministic.
2. A module built of pieces that each infer infers.
3. A refusal points at a non-empty span that lies within the source.
4. Every expression of an inferred program has a type.
5. No expression of a module whose signatures are written is left unsettled.
6. A generic function is general enough for any two uses of it.
7. A ring of declarations that hold one another by value is refused, naming the ring.
8. A chain of declarations that never comes back round is accepted.
9. `Option<()>`, written or inferred, is refused with `L0432`, and `Option<Bool>` there is accepted.
10. A type left out of a drawn signature is `L0436` there, and the help spells the settled one.

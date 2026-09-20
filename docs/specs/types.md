# Type inference

## Intent

Every expression of a module gets a type, and nothing in the module is written down twice to say so.
A signature is written where it documents a boundary; inside a function, inference does the work.
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

## Inference

A function is inferred from its signature inward.
A parameter with no written type gets a fresh type variable, and so does a missing result type.
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
Version 0.1 has no typeclasses, so it wires each operator to the instances the library will ship:

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

A function with no signature to read is inferred from its body, and is then generalised over the
variables that body left free.
Functions are inferred bottom up, which is the order a module is read in reverse.
A declaration sits below what uses it, which `docs/specs/modules.md` requires, so walking upwards
reaches a function before anything that calls it and a body with no signature has been given one
by the time a call reads it.

`total := count(users)` generalises what it binds, so a name bound this way is as polymorphic as
the value it was given.
`var total = 0` does not, because a mutable binding is assigned to later and must stay one type.

## The errors

| name              | code    | message                                          |
|-------------------|---------|--------------------------------------------------|
| type mismatch     | `L0400` | expected `Int`, found `String`                   |
| wrong arity       | `L0401` | `describe` takes 1 argument but 2 were given     |
| unknown field     | `L0402` | `User` has no field named `total`                |
| infinite type     | `L0403` | this would have a type that contains itself      |
| missing field     | `L0404` | `User` needs a field named `id`                  |
| field given twice | `L0405` | `User` is given `id` twice                       |
| not equatable     | `L0406` | `User` has no `Eq`, so two of them cannot be compared |
| zero divisor      | `L0407` | this divisor is zero, so there is no answer      |
| value discarded   | `L0408` | `Result<(), Error>` is left here and nothing takes it |
| unnamed arguments | `L0409` | `rename` gives two parameters the type `String`, so this call names its arguments |
| misnamed argument | `L0410` | this argument is named `to`, and the parameter here is `from` |
| no names to write | `L0411` | `Span` is a constructor, so it carries its values in order and names none |
| flag parameter    | `L0412` | this parameter is a `Bool`, so a call of `open` passes `true` and says no more |
| not a predicate   | `L0413` | `active` gives back a `Bool`, so its name asks the question it answers |
| holds itself      | `L0415` | `Node` holds `Node`                              |

`L0400` also says `` `Bool` cannot be added `` when `+` is given something that is neither `Int`
nor `String`.
`L0402` also says the type reached through `.` is not known, when inference never settled it.
It also says that a module is what was reached through, because nothing may reach inside one yet.
A variant that carries its values in order has no field to write against, so that is `L0402` too.
`L0401` counts the arguments of a written type as well as those of a call.
`L0407` is raised where a division is written with a `0` the compiler can already see.
`L0409`, `L0410`, and `L0411` are about how a call passes its arguments, which
`docs/specs/arguments.md` states.
All three are reached after `L0401` and after the arguments have met the parameter types, because
how many there are and what they are is each settled before which of them is which.
`L0411` also says a name was written on a call of the prelude, which declares none to check.
`L0412` is a parameter that is a bare `Bool`, which `docs/specs/arguments.md` states as well.
It is raised where the parameter is written, because the declaration is what changes.
It is reached after the body, because the type it reads is the one inference settled.
`L0413` is a function whose result is `Bool` and whose name asks nothing, which
`docs/specs/naming.md` states.
It reads the same result and is reached at the same point, after `L0412` and for the same reason.
A function whose parameters and result are all `Bool` is about `Bool`, and is the one carve-out.

`/` and `%` give back `Option<Int>` rather than `Int`, which `docs/specs/arithmetic.md` states.
`L0400` is what an `Option<Int>` met where an `Int` belongs is refused with, as anything else is.

Inference stops at the first error it reaches, which is the one lowest in the file.

## Properties

These hold and are checked with property-based tests:

1. Inferring a program never panics and is deterministic.
2. A module built of pieces that each infer infers.
3. A refusal points at a non-empty span that lies within the source.
4. Every expression of an inferred program has a type.
5. No expression of a module whose signatures are written is left unsettled.
6. A generic function is general enough for any two uses of it.
7. A ring of declarations that hold one another by value is refused, naming the ring.
8. A chain of declarations that never comes back round is accepted.

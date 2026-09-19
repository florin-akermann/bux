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

`Int` is a whole number, 64 bits wide, and it is the only one.
`()` is the type of a function that returns nothing interesting.
`List<T>` is opaque: version 0.1 has no syntax that builds one, only `for` that walks one.
`Option` and `Result` are ordinary algebraic data types supplied by the prelude.

A type the author writes is the type they get.
`fn wrap(raw: Int) -> UserId` is refused if its body returns an `Int`.
A written type carries one argument for each its declaration lists, so `List<Int, Int>` is refused.

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

The operators are fixed:

```text
+                        Int + Int, or String + String
- * / %                  Int
< <= > >=                Int
== !=                    two values of one type
&& ||  and prefix !      Bool
prefix -                 Int
```

An addition whose type is still unknown when its function has been inferred is an addition of
`Int`s.
This is the one default in the language, and it is here because version 0.1 has no typeclasses.

A block's type is the type of its last statement when that statement is an expression, and `()`
otherwise.
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
Functions are inferred in the order they are declared, so a call to a function declared later uses
that function's signature rather than its inferred type.

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

`L0400` also says `` `Bool` cannot be added `` when `+` is given something that is neither `Int`
nor `String`.
`L0402` also says the type reached through `.` is not known, when inference never settled it.
It also says that a module is what was reached through, because nothing may reach inside one yet.
A variant that carries its values in order has no field to write against, so that is `L0402` too.
`L0401` counts the arguments of a written type as well as those of a call.

Inference stops at the first error, as resolution does.

## Properties

These hold and are checked with property-based tests:

1. Inferring a program never panics and is deterministic.
2. A module built of pieces that each infer infers.
3. A refusal points at a non-empty span that lies within the source.
4. Every expression of an inferred program has a type.
5. No expression of a module whose signatures are written is left unsettled.
6. A generic function is general enough for any two uses of it.

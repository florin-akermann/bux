# Literals

## Intent

A whole-number literal takes the type its context expects, so a declared type is written as
plainly as `Int` is.

`docs/design.md` section 8 states the rule and leaves one question open: how an instance says
which whole numbers fit.
This spec answers it and states what inference, the fit check, and lowering each do.

Version 0.1 typed a literal at the one place it met one: `1` was an `Int`, always.
A declared `Int32` was therefore written `Int32(1)`, which is the wrapper `Int` never needs, and
`docs/principles.md` question 8 refuses a capability `Int` has and a declared type cannot.

Only a whole number is in question here.
A string and a truth value each have one type and no range, so neither has a trait and neither
reads any differently than it read before.

## The trait

```text
trait IntegerLiteral<T> {
    fn lowest() -> Int
    fn highest() -> Int
    fn from_literal(literal: Int) -> T
}
```

`from_literal` says how a whole number becomes a `T`, and it is what a literal is written as.
`lowest` and `highest` say which whole numbers a `T` holds, and they are what the fit check reads.

Three methods rather than one, because the two questions are answered at different times.
`from_literal` runs, and a program that never writes a literal never calls it.
`lowest` and `highest` are answered while the program is compiled, because a literal that does not
fit is a compile error and a compile error is never worked out at run time.

## A bound is read rather than run

The body of `lowest` and of `highest` is one whole number and nothing else, optionally with a `-`
in front of it.

The compiler reads those two bodies where it reads any other declaration, and it never runs them.
A body of any other shape is refused with `L0421`, where the body is written.
That is the whole of the restriction: no other method's body is read, and no expression is ever
evaluated at compile time.

The alternative was to run the two bodies while compiling, which is an evaluator for a language
that has one, and `docs/principles.md` refuses machinery no requirement asks for.
A range written into the `instance` header instead would be a second shape of instance that only
this trait has, and section 8 gives every trait one shape.

A bound is what the author claims their type holds, not what the machine can hold.
`Int32` below carries an `Int`, as every declared type carries what it declares; the range is the
domain its author means it to have, and the compiler holds every literal to it.

## What the compiler supplies

The prelude is not Lumen source yet, which `docs/specs/modules.md` states, so the compiler
declares the trait and the one instance the library will ship:

```text
instance IntegerLiteral<Int>
```

Its bounds are the whole numbers an `Int` holds, so every literal the lexer accepts fits `Int`.
`docs/specs/lexer.md` refuses a number that does not, as `L0103`, before any of this is reached.
That instance is supplied rather than written for a second reason: the lowest `Int` is one more
than the largest number the lexer reads, so no Lumen source could write its `lowest`.

`IntegerLiteral` and its three methods are ordinary prelude names rather than keywords, so a
module declaring one of them is refused with `L0302`.
A module writing `instance IntegerLiteral<Int>` is refused with `L0308`, because there already
is one.

## A declared type taking a literal

```text
instance IntegerLiteral<Int32> {
    fn lowest() -> Int {
        -2147483648
    }

    fn highest() -> Int {
        2147483647
    }

    fn from_literal(literal: Int) -> Int32 {
        Int32(literal)
    }
}

type Int32 = Int32(Int)
```

`let count: Int32 = 5` is then written, and so is `add(one: 1, other: 2)` where `add` takes two
`Int32`s.
Nothing else is needed: a literal is a trait method, and an instance is how a type gets one.

## What the context is

A literal's type is a variable, and it is settled by whatever the surrounding code says it is.

That is the same inference the rest of the language has, and a literal is no exception to it: a
declared result, a parameter's type, a field's type, and the other side of an operator each say
what the literal is, because each of them already says what everything else is.

The variable is not an ordinary one: it stands for a whole number, so it settles only on a type a
whole number may be written at, which is a type with an instance of `IntegerLiteral`.
Two of them meeting are one of them, and whatever settles the one that is left settles both.
A literal nothing settles is an `Int`, which is the one default the language keeps, and it reads
as `Int` wherever it is reported while nothing has settled it.
`1 + 1` is therefore an addition of `Int`s, and so is `var total = 0` in a function that says no
more about `total`.

A whole number meeting a type that takes none is the ordinary clash of two types, and `L0400`
reports it as one, at the place the two met.
`fn is_open() -> Bool { 1 }` reads `expected `Bool`, found `Int``, which is what it read before a
literal had a type of its own, and is what a reader of that line wants to be told.
Naming the missing instance instead would be true and unhelpful: `Bool` will never have one, so
"write the instance" is advice no reader should take.

A whole number settles on a named type, so it never settles on a type parameter however that
parameter is constrained, and `fn identity<T>(value: T) -> T { 1 }` is the `L0400` it always was.
Bounds are stated per instance, and a parameter stands for no one instance, so a literal held to
a parameter would be a literal held to no range at all.
A generic that wants a whole number calls `from_literal` by name, which is an ordinary trait
method and reads like every other constrained call.

## What is a literal, and what is not

A whole number written as an expression is a literal, and that is the whole of the rule.

A whole number written as a pattern is an `Int`, as it was before this.
`match count { 5 => … }` over an `Int32` is therefore refused with `L0400`.
A pattern asks whether two values are the same, which is `Eq` rather than `IntegerLiteral`, and
the trait a pattern needs is a separate question from the trait a literal is.
Nothing here needs the two answered together, so this answers the one it is about.

`from_literal` is an ordinary trait method, so a program may call it by name.
`from_literal(5)` at `Int` is the whole number it is given, and at a type whose instance a module
wrote it is that instance's method, exactly as the literal `5` at each of those is.

## The fit check

A literal is held to the bounds of the instance the type it settled on has, where it is written.
Unification has already settled that the type has one, so the range is all that is left to check.

`let count: Int32 = 5_000_000_000` is refused with `L0420`, and the message names the number, the
type, and the range that type holds.
The check runs once the literal's type has settled, so it is always about a type the reader can
see in the source rather than about a variable.

A literal that is the operand of a prefix `-` is checked as the negative number it reads as.
`-2147483648` at `Int32` fits, and `2147483648` at `Int32` does not, which is what a reader of the
range expects of the two.

## What is written

A literal at a type the compiler supplies the instance for is the number itself, which is what a
literal has always been.

A literal at a type whose instance a module wrote is an `invokestatic` of that instance's
`from_literal`, named as `docs/specs/traits.md` names an instance method:
`IntegerLiteral$Int32$from_literal`.
The number is pushed and the call takes it, so a literal costs one call and no more.

`lowest` and `highest` are written out like any other instance method, because every instance
method is written out whether anything calls it or not, which `docs/specs/traits.md` states.
Nothing calls these two: neither takes a value and neither gives one back that names the type, so
a call of one says nothing about which instance it means, and is refused as one at no type at all.
A bound is read rather than run, and that is the whole of what a bound is for.

## The errors

| Code | Raised when |
| --- | --- |
| `L0103` | A number does not fit in a whole number, which the lexer refuses first. |
| `L0302` | A module declares `IntegerLiteral`, or one of its three methods. |
| `L0308` | A module writes `instance IntegerLiteral<Int>`, which is already supplied. |
| `L0400` | A whole number is written at a type that takes none, including a type parameter. |
| `L0420` | A whole number does not fit the type it is written at. |
| `L0421` | A bound of an `IntegerLiteral` instance is not one whole number. |

```text
error[L0420]: `5000000000` does not fit `Int32`, which holds `-2147483648` to `2147483647`
error[L0421]: `lowest` of `Int32` is read rather than run, so it is one whole number
```

## Properties

These hold and are checked with property-based tests:

1. A literal nothing settles is an `Int`, whatever the module around it is.
2. A literal inside the range of the instance its type has is accepted, and one outside is refused.
3. A whole number written where an `Int` is expected infers and lowers exactly as it did before.
4. A literal at a type whose instance a module wrote calls that instance's `from_literal`.

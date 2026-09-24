# Arithmetic

## Intent

No Bux program throws, catches, or observes an exception, and no operation is partial.
An operation that has no answer for some of its input says so in its type rather than at runtime.
Division has no answer when the divisor is zero, so division says so in its type.

`docs/design.md` section 5 states the rule; this spec states what each operator does.

## The operators

Every operator is a trait method, and this spec states the instances the library ships for `Int`.
`docs/design.md` section 8 names the traits; a declared type writes its own instances the same way.
Version 0.1 wires `Int` and `String` to the operators directly, and behaves as the instances will.

`+` joins two `Int`s or two `String`s, and gives back what it was given.
`-` and `*` take two `Int`s and give an `Int`.

`/` and `%` take two `Int`s and give an `Option<Int>`:

```text
/   (Int, Int) -> Option<Int>
%   (Int, Int) -> Option<Int>
```

A divisor that is not zero gives `Some` of the answer.
A divisor that is zero gives `None`, because there is no whole number to give.

`17 / 5` is `Some(3)` and `17 % 5` is `Some(2)`; division truncates toward zero.
`17 / 0` and `17 % 0` are both `None`.

The type is what carries the news.
A reader of `total / count` sees an `Option<Int>` and knows the operation can have no answer,
without knowing anything about `count`.
Nothing is checked at runtime that the type did not already say.

## Why only `/` and `%`

`+`, `-`, and `*` are total, and their `Int` instances give an `Int`.
Two whole numbers always have a sum, and when it does not fit, the answer wraps.
Wrapping is a defined answer for every input, which is what makes the operation total.

`/` is different in kind rather than in degree.
There is no whole number equal to `x / 0`, so there is no answer to define.
An operation without an answer for some input says so in its type; that is the whole rule.

## Why `Option` and not `Result`

`Option<T>` says there is no answer.
`Result<T, E>` says the attempt failed and hands back something about how.

A zero divisor has one cause, and the caller is holding it.
An error value would be a type with one inhabitant, carrying nothing the caller does not have,
and `Result<T, ()>` is `Option<T>` written the long way round.

`?` also decides it.
A `Result` propagates into the error type of the function it is written in, so a division
returning one would make every function that divides declare an error type for it, and would
need a way to turn one error type into another.
An `Option` propagates into an `Option`, and costs none of that.

The rule is not about division.
Every operator that can fail gives back an `Option` or a `Result`, and which one is settled the
same way: `Option` when the absence explains itself, `Result` when the failure has something to
say that the caller could not work out.
`docs/design.md` section 5 states it for the language.

## Getting the answer out

An `Option` comes apart by `match`, or by `?` in a function that gives back an `Option`.
`(a / b)?` is the quotient where the divisor is not zero, and the function's `None` where it is.
That keeps arithmetic readable, because `?` marks the one operation that can have no answer:

```text
fn scaled(total: Int, count: Int) -> Option<Int> {
    Some(((total / count)? / 2)? + 2 * 5)
}
```

Each `/` gives an `Option<Int>` and carries its own `?`; `+` and `*` give an `Int` and carry none.
The body says which answer it gives, as `Ok(user)` does in a function that gives back a `Result`:
nothing is wrapped for the author, because `?` is the only thing here that is not written out.
In a function that gives back a `Result`, `(a / b)?` is refused, because a `None` names no error.
A `match` there states which error a zero divisor is, and there is no shorter way to say it.
The prelude supplies `or` for the case where a fallback is what the author means:

```text
or(maybe: Option<T>, fallback: T) -> T
```

`or(total / count, 0)` is the whole of the common case, and it is an ordinary call.
Both of its arguments are worked out before either branch is taken, because that is what a call
does, and what `or` does must not change when the prelude becomes Bux source.

The prelude never gains `unwrap` or `expect`.
There is no way to turn a `None` into a crash, which is the point of the rule.
Rust's `unwrap_or` shares its stem with a partial function it has nothing in common with, so the
total default is named for what it does instead.
`Result` has no `or` of its own yet: two functions of one name wait on typeclasses, which version
0.1 leaves out.
A `Result` comes apart by `match` or by `?` until then.

## What is generated

Neither `/` nor `%` lowers to a bare `ldiv` or `lrem`, because both throw on a zero divisor.
Each lowers to a test of the divisor, which yields `None` when it is zero and the quotient or the
remainder wrapped in `Some` when it is not.
`?` on an `Option` lowers as `?` on a `Result` does: a test of the tag, which gives the `None` it
was handed back as the function's answer, and otherwise reads the value out of the `Some`.
The `None` goes back exactly as it arrived, carrying nothing that would have to be built again.
Which of the two a `?` is about is settled by the kind its function gives back.
What the `?` is written on answers where no signature has, and a `?` neither of them settles waits
for the body, which is the last thing that can say what a function gives back.

No method a module writes can throw.
The one `athrow` code generation emits sits after an exhaustive `match`, where every value of the
type has already been answered, so nothing reaches it.

## The errors

A divisor written as `0` is refused with `L0407` rather than compiled into a `None`.
The compiler can see that division has no answer, so the program is refused where it is written:

```text
error[L0407]: this divisor is zero, so there is no answer
help: a zero written here is never anything else; drop the division
```

`total / 0` is that; `total / count` is not, because nothing here says what `count` holds.

Every other refusal is an ordinary mismatch.
`Int` met where `Option<Int>` belongs is `L0400`, as any other mismatch is.
`total / count + 1` is refused for that reason: `+` joins two `Int`s, and the left is an `Option`.
`(total / count)? + 1` is what an author who meant to propagate the `None` writes.
A `Result` function refusing `(total / count)?` is `L0400` too, because `Option` is not `Result`.

## Properties

These hold over any module that divides, and drawn properties, each a test of `tests/`, check them:

1. Every `/` and `%` tests its divisor before it divides, whatever it is written over.
2. Every `/` and `%` builds both answers a divisor can have: a `Some` and a `None`.
3. `or` is written out where it is used, calling nothing of its own and reading both branches.
4. Every `?` on an `Option` gives the `None` it was handed back unchanged, wherever it is written.

`tests/spec/arithmetic/division.bx` runs the arithmetic this spec states on a JDK, and is
skipped when none is present.
It divides through `?` as well as through `or` and `match`, so the propagation runs there too.
It writes out what each division worked out, and its header states the lines it must write.
Each answer is checked rather than asserted, in the form `docs/specs/executable-examples.md` gives.

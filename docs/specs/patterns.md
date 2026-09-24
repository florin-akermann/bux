# Patterns

## Intent

A pattern says what a value has to be, and binds the parts of it the arm goes on to use.

Version 0.1 wrote two of them: a bare name, and a constructor with patterns or field names inside.
`docs/implementation.md` section 10 promises richer pattern matching and does not say richer how.
Three forms earn their keep in everyday code, and this spec is the whole of the answer.

`_` ignores a value, a literal matches one value, and `A | B` answers two variants in one arm.
Each is a pattern like any other: it is written wherever a pattern is written, nested included.

## `_` matches anything and binds nothing

```text
match status {
    Failed(reason) => reason
    _ => "fine"
}
```

A name that binds does the same matching, so `_` earns its keep by what it does not do.
A binding the arm never reads is a name the reader looks for a use of and does not find, and
`docs/specs/naming.md` holds every declared name to being worth looking up.
`_` says the value is not read, which is the one thing a name cannot say.

It is the same `_` that `docs/specs/discarding.md` writes on the left of `=`, and it says the same
thing in both places: what is here is deliberately not used.

A binding that nothing reads is `L0321`, which `docs/specs/unused.md` states.
`Box {}` matches a record of type `Box` and binds no field, so an arm that needs no field writes it.
It binds nothing, so it can be one alternative of an or-pattern.
A variant that carries its values in order has no field, so `Some {}` is `L0401`, as in a build.
A variant that carries nothing is matched by `Dot {}` as by `Dot`, as an expression builds it.

## A literal matches one value

```text
match count {
    0 => "none"
    1 => "one"
    _ => "some"
}
```

A whole number, a string, and a truth value are each written as a pattern.
A pattern asks whether two values are the same, so the type it is written over needs `Eq`.

A whole number takes the type it is matched against, exactly as one written as an expression does.
`docs/specs/literals.md` states that rule for an expression, and a pattern is held to the same one.
So `match count { 5 => … }` over an `Int32` asks `IntegerLiteral` for what `5` is at that type, and
`Eq` for whether the two are the same.
A type with neither is the `L0400` it always was, and one with the first and not the second is
`L0418`, which names the instance that is missing.

The type a literal is matched against is the one the place it is written holds, nested included:
`Box(1)` over a `Box<Int>` is matched against the `Int` the variant carries.

A string and a truth value each have one type and no range, which `docs/specs/literals.md` states,
so a string pattern is a `String` and `true` is a `Bool`, and neither takes a type from anywhere.

A literal names one value of a type that has more, so a `match` that writes one needs an arm that
covers the rest: a `_` or a name that binds, or it is `L0500`.

## An or-pattern answers two alternatives in one arm

```text
match status {
    Pending | Running => "in flight"
    Failed(reason) => reason
    Done => "done"
}
```

`A | B` matches what `A` matches and what `B` matches, and an arm that writes one is one arm.
Alternatives are tried in the order they are written, and the first that matches answers.

**An or-pattern binds nothing.**
Every alternative of it is a pattern that binds no name, and one that binds is `L0314`.
`Failed(reason) | Cancelled(reason)` would need every alternative to bind the same names at the
same types, which is a rule about the alternatives rather than about each of them, and no example
in hand asks for it.
The arms it would save are written as two arms, which is what `docs/principles.md` question 9 asks
of any second way to write something.

`_` binds nothing, so `0 | _` is written and means what `_` means.
An alternative is a whole pattern, so `Failed(0 | 1)` writes one inside a constructor.

Alternatives are held to the order their type declares them, which is the rule every arm is held
to: `docs/design.md` section 13 asks for it so that a new variant has one place to be handled.
`Running | Pending` is therefore `L0501` where the type declares `Pending` first.
The arm itself is placed by its first alternative, so an arm answering for the first and the last
of three variants is written above the arm answering for the middle one.

## A guard is not one of them

A guard is a condition written after a pattern, as `n if n > 0 =>` writes one in other languages.
It is refused.

An `if` inside the arm reads the same and says the same thing, so a guard is a second way to write
what the language already writes, which `docs/principles.md` question 9 refuses.
Exhaustiveness is the other half of the answer: a guard is an expression, and no check can say
whether the arms of a `match` whose patterns are guarded cover every value.
A language with guards either runs the check as though the guard were not there, which reports
arms it cannot prove, or gives up on it; neither is what `docs/design.md` section 4 asks for.

## Canonical form

`_` is written as itself, and an alternative is separated by one space, a `|`, and one space.
Nothing brackets an or-pattern: `A | B` is the whole of it, and `(A | B)` is not written.
A pattern stays on one line, as every pattern does, which `docs/specs/formatting.md` states.

The formatter keeps what the author wrote in each of the three forms: none of them is a shorthand
for another, so there is nothing to rewrite and nothing to prefer.

## The errors

| code    | what it refuses                                              |
| ------- | ------------------------------------------------------------ |
| `L0314` | a name that binds is written inside an or-pattern             |
| `L0400` | a literal pattern over a type it is not a value of            |
| `L0418` | a literal pattern over a type with no `Eq`                    |
| `L0500` | the arms do not cover every value, which a literal never does |
| `L0501` | an arm or an alternative is written out of declaration order  |

`L0314` reads:

```text
error[L0314]: `reason` binds inside an or-pattern, which binds nothing
  --> demo.bx:3:12

  3 |     Failed(reason) | Cancelled(reason) => reason
    |            ^^^^^^

help: write one arm for each alternative where one of them binds
```

## Properties

These hold and are checked by drawn properties in the runner:

1. A `match` whose arms the check accepts is one every value of the type reaches an arm of.
2. `A | B` in one arm covers exactly what `A` and `B` cover in two arms.
3. Canonical form leaves each of the three forms as the author wrote it.

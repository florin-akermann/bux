# Exhaustiveness

## Intent

A `match` answers for every value it may meet.
`docs/design.md` section 4 asks for this, and gives the reason: adding a variant must make the
compiler point at every match that has not been told what to do with it.
A match that leaves a value unanswered is refused, and the refusal names a value it does not cover.

The phase consumes the typed tree and yields nothing.
It is a check, as canonical form is: it either has nothing to say, or it refuses the file.
It runs after inference because it relies on inference's guarantee that every pattern of one match
is a pattern of the scrutinee's type.

## What covers what

A pattern that is a bare name binds, and binding covers every value of the type.
Every other pattern names a constructor, and covers the values that constructor builds whose
carried values its own patterns cover.

A type's constructors are the ones its declaration writes:

```text
a record type          the one constructor of the record's own name
an algebraic data type one constructor per variant
Bool                   true and false
Option<T>              Some and None
Result<T, E>           Ok and Err
Int, String            more than a match can write down
```

A match covers a type when it covers every constructor of it.
`Int` and `String` have more values than a match can list, so only a name that binds covers them.
`List<T>` has no pattern that builds one, so only a name that binds covers it too.

The arms are read together rather than one at a time.
`Some(Ok(value))`, `Some(Err(problem))`, and `None` cover an `Option<Result<T, E>>` between them,
though no one of them covers it alone.

A record pattern names the fields it binds and leaves the rest alone, so it covers every value its
constructor builds.
`Authorized { authorization_id }` covers every `Authorized`.

## The witness

A refusal names a value the match does not cover, written in the shape of an arm that would
cover it.
`_` stands there for any value, which an arm writes as a name of its own: version 0.1 has no
pattern that matches without binding.
`_` is reserved for the discard of `docs/specs/discarding.md` and is no pattern at all, so a
witness is read and answered rather than pasted back into the file.

```text
error[L0500]: this `match` does not cover `Failed(_)`
```

Every uncovered value is named, in the order the declaration writes the constructors, so that one
reading of the file says everything the match is missing.

```text
error[L0500]: this `match` does not cover `Authorized(_)`, `Failed(_)`
```

The refusal points at the whole match expression, because that is what is incomplete; no one arm
of it is wrong.

Checking stops at the first match it refuses, as inference does.

## The order of the arms

A `match` lists its arms in the order the type declares its variants.
`docs/design.md` section 13 asks for it so that a new variant has exactly one place to be handled,
and so that no diff is ever reorder-only.

The rule is checked here because this is the phase that knows the variant list.
An arm written above one the type declares above it is refused, pointing at the arm out of place.

```text
error[L0501]: this `match` writes `Pending` after `Failed`
```

An arm is placed by the constructors it writes, read left to right and outermost first.
Two arms that reach inside one variant therefore answer to that variant first and to what they
reach for second: `Some(Ok(_))` is written above `Some(Err(_))`, and both above `None`.

Placing stops at the first thing no declaration writes down.
A name that binds, a number, and a string are each written where the author put them, and so is
everything the arm writes after one of them.
`true` and `false` are literals too, so the two values of a `Bool` are written either way,
whether they stand alone or inside a variant.

Coverage is decided first: a `match` that leaves a value unanswered is refused for that, because
adding the missing arm is what settles where the arms go.

## The errors

| name                 | code    | message                                          |
|----------------------|---------|--------------------------------------------------|
| non-exhaustive match | `L0500` | this `match` does not cover `Failed(_)`          |
| arm out of order     | `L0501` | this `match` writes `Pending` after `Failed`     |

## Properties

These hold and are checked with property-based tests:

1. Checking a typed program never panics and is deterministic.
2. A match whose arms cover every constructor of its scrutinee, in order, is accepted.
3. A match with any one of its arms removed is refused, when every arm named a constructor.
4. A match with an arm that binds a name is accepted, whatever else it writes.
5. A refusal points at a non-empty span that lies within the source.
6. A match whose arms are rotated out of the declared order is refused.

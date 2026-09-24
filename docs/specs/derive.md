# Derive

## Intent

A record or a variant derives a standard trait, so the instance the author would have written by
hand is the instance the compiler writes.

`docs/design.md` section 8 promises `derive Eq, Ord, Hash for User` and refuses `==` on a declared
type until that type has an instance.
Equality is by state, which `docs/design.md` section 2 makes the whole of what a value is, so the
instance follows from the declaration and there is nothing for an author to decide.

All four of the standard traits are derivable: `Eq`, `Ord`, `Hash`, and `Show`.
`docs/specs/traits.md` states what each one promises; each promise is a reading of what a value
holds, so each follows from the declaration and there is nothing for an author to decide.

No other trait is derivable, and a derive of one is refused by name, so the reader is told which
is which.

## The form

```text
derive Eq for User

type User = {
    id: Int
    name: String
}
```

`derive` is a keyword, and a derive is a top-level item like an import, a type, or an instance.
It names one or more traits, then `for`, then one type this module declares.

```text
derive  = "derive" Name { "," Name } "for" Name
```

The list is written with `, ` between two names and no trailing comma, and the whole item is one
line however long it is, because it holds nothing that could be broken across lines.

A derive names the type below it, as every use in Bux names a declaration below it, so
`docs/specs/naming.md` puts `derive Eq for User` above `type User`.

## What it writes

A derive writes the instance, and an instance is an instance: `docs/specs/traits.md` states what
one is and what one is written as, and a derived one is written as that and nothing else.

So `derive Eq for User` writes `Eq$User$is_equal` as a static method of the module class, taking
two `User`s and giving back a `Bool`, exactly where an `instance Eq<User>` would have written it.
`one == other` over two `User`s is then the `invokestatic` any instance's method is.
Nothing else about `==` changes, and nothing can tell the two instances apart.

Each derive reads the declaration the same way: a record is read field by field in the order it
declares its fields, and a variant is read by which variant it is and then by what that variant
carries, in the order it carries them.
Each field and each carried value is handed to the instance of its own type, which is the one rule
the whole thing rests on: `Int` and `String` by what the prelude supplies, and a declared type by
its own instance, derived or written.
A field of type `List<T>` is handed to the prelude's instance over `List<T>`, which hands each
element to the instance of `T`, so a record holding a `List<Int>` derives all four.
Nothing asks whether two references are one object, which `docs/specs/codegen.md` requires.

### `Eq`

Two values are equal when they are the same variant and everything they hold is equal.
A variant that carries nothing is equal to itself, so two `Pending`s are one value.

The comparison stops at the first field that differs, because `&&` stops there and a derived
instance is the instance an author would have written with `&&`.

### `Ord`

A variant comes before another when the type declares it first, so `Pending < Failed("late")`
whatever the failure says.
Two values of one variant are ordered by what they carry, the first value that differs deciding,
which is the order a reader reads the declaration in.

Two records are ordered by their fields, the first field that differs deciding.
A record with no field, and a variant that carries nothing, is ordered before nothing and after
nothing: it is equal to itself, which the total order asks.

### `Hash`

A value hashes to a whole number worked out from the hash of each value it holds, in the order it
holds them, and, where the type declares variants, from which variant the value is.
Equal values therefore hash alike, which is the whole of what `Hash` promises.

The number two unequal values work out to is not part of this spec, and nothing may be written
that depends on it.

### `Show`

A value is shown as the source that builds it.

```text
User { id: 1, name: ada }
Pending
Failed(late)
Sent { to: Where { id: 7 }, note: hi }
```

A record is its type's name, then its fields in declaration order, each written `name: value`,
joined by `, ` inside `{ }`.
A variant that carries nothing is its own name.
A variant carrying values in order is its name and those values inside `( )`, joined by `, `.
A variant carrying fields is its name and those fields inside `{ }`, as a record's are.

How a value is written follows from how its type is declared and not from how much it holds, so a
record or a variant that declares braces and no field inside them is written `Empty {}`.

Each value is shown by the `Show` of its own type, so a `String` field shows its characters
without quotes, which `docs/specs/traits.md` states.

## What a type needs to derive

Every field of a record, and every value every variant carries, has a type that has the trait
being derived.

A type that does not is refused with `L0422`, which names the trait, the value, and the type it
is: a derive is a promise about the whole value, and a value is only as comparable, as hashable,
or as showable as what it holds.
A type written with arguments has the instance its own head has, read at the arguments it is
written with, which `docs/specs/traits.md` states.
So `List<Int>` has all four and `List<Crate>` has none until `Crate` does, and `L0422` fires where
the element has none rather than wherever a list is held.
`()` has no instance of anything, so a field of that type is refused whatever the trait is.

Each trait is read on its own, so `derive Ord for User` asks every field for `Ord` and asks
nothing for `Eq`; `derive Eq, Ord for User` writes both and is the usual way to ask.

The check reads the instances the module has, so a field whose type derives the trait further down
the file counts as having it, and a variant and a record that hold each other derive it together.
A field of a type of another module has the instances that travel with that type.
So `derive Eq for Order` over a field of `demo.User` is accepted where `demo` gives `User` an `Eq`.
Two records that hold each other never reach the check: `docs/specs/types.md` refuses a ring of
records with `L0415` before any derive is read, because no such value could be built.

## The errors

| Code | Raised when |
| --- | --- |
| `L0300` | A derive names a trait or a type that nothing declares. |
| `L0303` | A derive is written below the type it names. |
| `L0308` | A derive writes an instance the module already has, written or derived. |
| `L0310` | A derive names something that is not a trait. |
| `L0311` | A derive names something that is not a type. |
| `L0312` | A derive names a trait no type derives, or a type this module does not declare. |
| `L0401` | A derive names a type that takes arguments, which a derive never names. |
| `L0422` | A type derives a trait and something it holds has no instance of it. |

```text
error[L0312]: `Add` is not a trait a type derives
error[L0312]: `Int` is not a type this module declares
error[L0422]: `User` derives `Hash`, and the `List<Crate>` it holds as `tags` has none
```

`L0406` says a type has no instance where an operator is written over it, and now says what to do
about it: its help reads `` `User` gets one by deriving it: write `derive Eq for User` ``, naming
the trait the operator wanted.

A value a variant carries is named for its variant and for its own place in it, so the second value
of `Failed(String, List<Int>)` is `Failed.1` and the `tags` of a `Sent { tags: List<Int> }` is
`Sent.tags`.

`L0300`, `L0303`, `L0308`, `L0310`, `L0311`, and `L0312` are raised by name resolution, which
`compiler/resolver.bx` words.
`L0401` and `L0422` are raised by type inference, which `compiler/refusal.bx` words.

## Executable examples

`tests/spec/traits/derived.bx` runs a record and a variant that each derive `Eq`, which is where
the claim that a derived `Eq` agrees with the state of two values is held to a running program.
`tests/spec/traits/derived_ord.bx`, `tests/spec/traits/derived_hash.bx`, and
`tests/spec/traits/derived_show.bx` do the same for the other three.
`tests/spec/traits/derived_holds_no_instance.bx` and `tests/spec/traits/derive_not_derivable.bx`
are the two refusals; the first holds a `List<Crate>`, whose element has no instance.
`tests/spec/traits/over_a_list.bx` runs a record that holds a `List<Int>` and derives `Eq`, which
is where the claim that a derive reaches through a list is held to a running program.

## Properties

These hold and are checked by drawn properties, each a test of `tests/`:

1. A derive survives printing and parsing unchanged, and canonical form is idempotent over one.
2. A record of any shape whose every field has the trait derives it, whichever of the four it is,
   and one with a field that has none is refused naming that field and the trait.
3. A record of any shape derives a body that reads every field in the order the type declares
   them, and hands each to the instance of that field's own type: `Eq` reads both values of a
   field, `Ord` reads them both ways round, and `Hash` and `Show` read the one they are handed.

What each derived instance then answers is a claim about a running program, so it is held to by
the executable examples above rather than by a property: `derived_ord.bx` holds the order to
trichotomy and transitivity over the values it names, and `derived_hash.bx` holds equal values to
hashing alike.

That a derived instance is reached the way a written one is, so that the two lower to the same
call, is likewise held to by an example: there are two ways a type comes by an instance and
nothing to generate.

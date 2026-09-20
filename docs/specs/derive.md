# Derive

## Intent

A record or a variant derives `Eq`, so the instance the author would have written by hand is the
instance the compiler writes.

`docs/design.md` section 8 promises `derive Eq, Ord, Hash for User` and refuses `==` on a declared
type until that type has an instance.
Equality is by state, which `docs/design.md` section 2 makes the whole of what a value is, so the
instance follows from the declaration and there is nothing for an author to decide.

Only `Eq` is derivable here.
`Ord`, `Hash`, and `Show` are named in section 8 and land with the traits themselves; a derive of
one of them is refused today, and refused by name, so the reader is told which is which.

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
derive := "derive" Name { "," Name } "for" Name
```

The list is written with `, ` between two names and no trailing comma, and the whole item is one
line however long it is, because it holds nothing that could be broken across lines.

A derive names the type below it, as every use in Lumen names a declaration below it, so
`docs/specs/naming.md` puts `derive Eq for User` above `type User`.

## What it writes

A derive writes the instance, and an instance is an instance: `docs/specs/traits.md` states what
one is and what one is written as, and a derived one is written as that and nothing else.

So `derive Eq for User` writes `Eq$User$is_equal` as a static method of the module class, taking
two `User`s and giving back a `Bool`, exactly where an `instance Eq<User>` would have written it.
`one == other` over two `User`s is then the `invokestatic` any instance's method is.
Nothing else about `==` changes, and nothing can tell the two instances apart.

A record is equal field by field, in the order the type declares its fields.

A variant is equal when both values are the same variant and what that variant carries is equal,
value by value, in the order the variant carries them.
A variant that carries nothing is equal to itself, so two `Pending`s are one value.

Each field and each carried value is compared by the `Eq` of its own type, which is the one rule
the whole thing rests on: `Int` by the whole numbers it holds, `String` by its characters, and a
declared type by its own instance, derived or written.
Nothing asks whether two references are one object, which `docs/specs/codegen.md` requires.

The comparison stops at the first field that differs, because `&&` stops there and a derived
instance is the instance an author would have written with `&&`.

## What a type needs to derive `Eq`

Every field of a record, and every value every variant carries, has a type that has `Eq`.

A type that does not is refused with `L0422`, which names the value and the type it is:
a derive is a promise about the whole value, and a value is only as comparable as what it holds.
`()` has no `Eq`, and neither has a type written with arguments such as `List<Int>`, because
`docs/specs/traits.md` gives an instance to a type written by name.

The check reads the instances the module has, so a field whose type derives `Eq` further down
the file counts as having one, and a variant and a record that hold each other derive it together.
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
| `L0401` | A derive names a type that takes arguments, which no instance names. |
| `L0422` | A type derives `Eq` and something it holds has none. |

```text
error[L0312]: `Ord` is not a trait a type derives
error[L0312]: `Int` is not a type this module declares
error[L0422]: `User` derives `Eq`, and the `List<Int>` it holds as `tags` has none
```

`L0406` says a type has no `Eq` where `==` is written over it, and now says what to do about it:
its help reads `` `User` gets one by deriving it: write `derive Eq for User` ``.

A value a variant carries is named for its variant and for its own place in it, so the second value
of `Failed(String, List<Int>)` is `Failed.1` and the `tags` of a `Sent { tags: List<Int> }` is
`Sent.tags`.

`L0300`, `L0303`, `L0308`, `L0310`, `L0311`, and `L0312` are raised by name resolution, which
`crates/resolver/src/error.rs` words.
`L0401` and `L0422` are raised by type inference, which `crates/types/src/error.rs` words.

## Executable examples

`tests/spec/traits/derived.lm` runs a record and a variant that each derive `Eq`, which is where
the claim that a derived `Eq` agrees with the state of two values is held to a running program.
`tests/spec/traits/derived_holds_no_instance.lm` and `tests/spec/traits/derive_not_derivable.lm`
are the two refusals.

## Properties

These hold and are checked with property-based tests:

1. A derive survives printing and parsing unchanged, and canonical form is idempotent over one.
2. A record of any shape derives an `Eq` that reads every field of both values, in the order the
   type declares them, and compares each by the `Eq` of that field's own type.
3. A record of any shape whose every field has `Eq` derives it, and one with a field that has
   none is refused naming that field.

That a derived instance is reached the way a written one is, so that the two lower to the same
call, is stated where it belongs above and held to by an example rather than by a property: there
are two ways a type comes by `Eq` and nothing to generate.

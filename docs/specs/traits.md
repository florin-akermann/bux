# Traits and instances

## Intent

A trait names what a type can do, and an instance is one type doing it.

`Eq` is the first: `==` compares two values only when their type has an instance of `Eq`, and a
type the JVM holds is no exception to that.
`docs/design.md` section 8 states the design; this spec states what is written, what is refused,
and what a constraint resolves to.

A constraint resolves while the program is compiled.
A generic is written once per set of types it is used at, which `docs/specs/codegen.md` states, so
the body written for a set of types calls the instances those types have.
Nothing is passed at run time, no method table is built, and no instance is looked up.

## Declaring a trait

```text
trait Eq<T> {
    fn is_equal(one: T, other: T) -> Bool
}
```

A trait declares a name, one type parameter, and one or more method signatures.
A signature is a function's first line and nothing else: a name, its parameters, and its result.
It has no body, because a trait says what a method is and an instance says what it does.

A signature writes the type of every parameter it takes.
A function may leave one out and let inference read it off the body; a signature has no body, so a
type left out of one is a type nothing would settle, and `L0419` refuses it where it is missing.
A method that gives nothing back writes no `->`, which is the one thing a signature may leave out.

One type parameter is what every trait the language has asks for.
`Eq<T>`, `Ord<T>`, `Show<T>`, and `IntegerLiteral<T>` each name one type and say what it can do.
A trait over two types is a question for the spec that needs one.

A trait's name is a name in the type scope, beside the types, because that is where it is written:
in `instance Eq<Point>` and in `T: Eq<T>`.
Each of its method names is a name in the value scope, beside the functions, because that is where
a method is called: `is_equal(one, other)`.
A trait is therefore refused where a type belongs, and a type is refused where a trait belongs.

A method's signature is written over the trait's type parameter, and it is the one type parameter
a signature may name.

## Giving a type an instance

```text
instance Eq<Point> {
    fn is_equal(one: Point, other: Point) -> Bool {
        one.across == other.across && one.down == other.down
    }
}
```

An instance names a trait, the type it is for, and a body for each of the trait's methods.

The type is written by name, without arguments: `Eq<Point>`, and not `Eq<List<Point>>`.
A type that takes arguments is refused here, counted as a written type is counted anywhere else:
`instance Eq<Option>` names one of the arguments `Option` takes and would answer for every one.
An instance for a generic type waits for the spec that derives one.

The name is a type, so a trait written there is refused with `L0311`, as one written where any
other type belongs is.

An instance writes a body for every method its trait declares, once, and for no other name.
A method it leaves out, a name the trait never declared, and a second body for one method are each
refused: an instance's method names are its trait's, so a name written twice would put no second
name in scope and one of the two bodies would be reached while the other was lost.

Each body's signature is the trait's, with the trait's type parameter standing for the instance's
type, so `is_equal` in `instance Eq<Point>` takes two `Point`s and gives back a `Bool`.
A body may leave a type out and let inference read it off the trait, so `fn is_equal(one, other)`
is the same declaration written shorter.
A body that writes a type the trait does not have is a mismatch, reported where it is written.

An instance declares no name.
Two instances each write `is_equal`, and neither of them is the `is_equal` anything calls: the trait
declares that name, and the instances say what it does at each type.

An instance is written in the module that declares the trait or in the module that declares the
type it is for.
Version 0.2 has one module reach another's functions and nothing more, so both are this module,
and the rule is stated here for the change that makes the two differ.

One trait and one type have one instance.
A second instance of one trait for one type is refused where it is written.

An instance is written by hand or by a derive, and the two are the same instance: what a derive
writes is what an author would have, and a second one either way is the second instance refused.
`docs/specs/derive.md` states which traits a type derives and what each derive writes.

## The standard traits

`docs/design.md` section 8 names four traits every language wants and every type may opt into:
`Eq`, `Ord`, `Hash`, and `Show`.
Each is a trait like any other, each declares one method, and no type has any of them until it
writes the instance or derives it.

```text
trait Eq<T> {
    fn is_equal(one: T, other: T) -> Bool
}

trait Ord<T> {
    fn is_less(one: T, other: T) -> Bool
}

trait Hash<T> {
    fn hashed(value: T) -> Int
}

trait Show<T> {
    fn shown(value: T) -> String
}
```

`Hash` is a trait a type opts into rather than the `hashCode` every JVM object is born with, and
`Show` is one it opts into rather than `toString`.
`docs/design.md` section 2 keeps a value free of both: a type that never asked has no hash to be
put in a map by and no text to be printed as, and asking is writing the instance or the derive.

### What each one promises

`Eq` says when two values are one value, and it reads what they hold.
Two values are equal when everything they hold is equal, and nothing asks whether two references
are one object.

`Ord` says which of two values comes first.
`is_less` is the one method the four comparisons are written with, which
`docs/specs/operators.md` states.
The order is total: for any two values exactly one of `one < other`, `other < one`, and
`one == other` holds, so `Ord` and `Eq` never disagree.
It is also transitive: where `a < b` and `b < c`, `a < c`.
The law is a law about a type that has both; a type may have `Ord` alone, and `docs/specs/derive.md`
asks nothing of `Eq` when it writes one.

`Hash` gives a value a whole number that stands for what it holds.
Equal values hash alike, which is the whole of what a hash promises; two values that hash alike
may still differ, and nothing reads a hash as an answer about equality.
The law binds a type that has both, as `Ord`'s does.

`Show` renders a value as text a reader reads.
It is a function of what the value holds, so two equal values are shown alike.
A `String` is shown as the characters it holds and nothing more: `shown("ada")` is `ada`, because
`Show` is one rule for every type and a quote would be a rule for one of them.

### The instances the prelude has for them

The prelude has all four for `Bool`, `Int`, and `String`:

```text
instance Eq<Bool>       instance Eq<Int>       instance Eq<String>
instance Ord<Bool>      instance Ord<Int>      instance Ord<String>
instance Hash<Bool>     instance Hash<Int>     instance Hash<String>
instance Show<Bool>     instance Show<Int>     instance Show<String>
```

All twelve are Lumen source, a few lines each, and `library/prelude.lm` is where a reader goes
to find what one of them says.
`Hash<String>` reads a string by the characters it holds, over a `String.hashCode` declared with
the width `docs/specs/interop.md` states.

None of the twelve has a body anything calls.
What one amounts to is written out where it is called — the JVM instruction for it, or a call of
the Java member behind it — which `docs/specs/library.md` states and `docs/specs/codegen.md`
writes out.

`false` comes before `true`, which is the order the two are written in and the order a JVM already
puts them in.
Two strings are ordered by their characters, one by one, and a string that begins another comes
first: `"a" < "ab" < "b"`.

`hashed` at `Int` is the whole number itself, at `Bool` is `0` for `false` and `1` for `true`, and
at `String` is a number worked out from the characters it holds.
Which number two unequal values work out to is not part of this spec, and nothing may be written
that depends on it.
`shown` at `Int` is the digits it is written with, at `Bool` is `true` or `false`, and at `String`
is the string.

`Eq`, `Ord`, `Hash`, and `Show`, and `is_equal`, `is_less`, `hashed`, and `shown`, are ordinary
prelude names rather than keywords, so a module declaring one of them is refused with `L0302`.

## Constraining a generic

```text
fn has_value<T: Eq<T>>(items: List<T>, value: T) -> Bool {
    for item in items {
        if is_equal(item, value) {
            return true
        }
    }
    false
}
```

A type parameter is written with the trait it is constrained by, which is what lets the body call
that trait's methods at it.

A type parameter takes one constraint.
The spec that puts a second trait on one parameter settles how the two are written.

A call of a trait method at a type parameter is accepted only where the parameter is constrained
by that method's trait.
A call of `has_value` is accepted only where what `T` settled on has an instance of `Eq`.
Neither is a run-time question: both are answered where they are written.

## How a constraint is resolved

A use of a trait method is resolved by the type its trait's parameter settled on:

- a named type reaches the one instance of that trait for that type;
- a type parameter of the function being compiled reaches the constraint that parameter declares;
- anything else is refused, because nothing names an instance.

A generic is compiled once per set of types, so the second case is a case only while type checking.
The body written for `has_value` at `Point` has `T` standing for `Point`, and the `is_equal` it
calls is the one `instance Eq<Point>` declares.
The instance `has_value` at `Int` resolves to is the one `library/prelude.lm` writes for `Int`,
and what that instance amounts to is written out in place rather than called.

`==` and `!=` are resolved the same way, against `Eq`.
A comparison of two values of a type with no instance of `Eq` is refused as it was before traits,
with `L0406`, and a comparison nothing settled is a comparison of `Int`s, which is the one default
the language keeps.

## What the library writes and what the compiler supplies

`library/prelude.lm` is Lumen source the compiler carries, which `docs/specs/library.md` states,
and it declares every trait and writes every instance of one for `Bool`, `Int`, and `String`.
The four standard traits are written out above; `docs/specs/operators.md` writes out the six the
arithmetic operators are, and `docs/specs/literals.md` writes out `IntegerLiteral`.

`todo` is the one function the compiler supplies, because a hole has no body for the library to
write.
`Bool`, `Int`, `String`, and `List` stay the compiler's as well, because what they are made of is
the JVM rather than a declaration, which `docs/specs/library.md` states.
No instance, trait, or function of the prelude other than `todo` is the compiler's.

A module writing `instance Eq<Int>` is refused with `L0308`, because there already is one, and a
module declaring its own `Eq` is refused with `L0302`, exactly as one declaring its own `todo` is.
Which of the two wrote the one already there makes no difference to either refusal.

No instance over those three types has a body anything calls.
`is_equal` at `Int`, at `Bool`, or at `String` is written out where it is called, as `or` is, and
the comparison it writes is the one `==` already wrote: two whole numbers or two truth values as the
JVM compares them, and two strings by the characters they hold.
The body in `library/prelude.lm` is what says in Lumen what that instruction does, and reading it
is what holds it to its type.
Nothing asks whether two references are one object, which `docs/specs/codegen.md` requires.

## What is written

An instance's method is a static method of the module class, named for its trait, its type, and
itself, joined by `$`: `Eq$Point$is_equal`.
`$` is legal in a JVM method name and Lumen writes no operator with it, which is the same reason
`docs/specs/codegen.md` names a specialized generic that way.

An instance's method is written whether anything calls it or not, as a function that declares no
type parameter is, because it names no type parameter of its own.

A call of a trait method is an `invokestatic` of the method the instance wrote, or, over a type
the JVM holds, the instruction that instance amounts to written out in place.
`one == other` over a type with an instance is that same call, and `one != other` is that call
with the answer flipped.

## Canonical form

A trait is written with one signature per line and no blank line between two of them, as a record
type is written with one field per line.

An instance is written with one function per line and one blank line between two of them, as a
module is written with one blank line between two items.

A trait and an instance each take the indentation and the brace placement every block takes, which
`docs/specs/formatting.md` states.

A trait's name and the type an instance is for are written in `PascalCase`, and a method's name in
`snake_case`, which `docs/specs/naming.md` already asks of a type and a function.

## The errors

| Code | Raised when |
| --- | --- |
| `L0302` | A module declares a prelude trait or method, such as `Eq` or `is_equal`. |
| `L0308` | A trait already has an instance for the type a second instance names. |
| `L0309` | An instance leaves a method out, writes an undeclared one, or writes one twice. |
| `L0310` | Something that is not a trait is written where a trait belongs. |
| `L0311` | A trait is written where a type belongs. |
| `L0400` | An instance's method has a signature the trait's method does not. |
| `L0401` | An instance names a type that takes arguments, which an instance never names. |
| `L0406` | An operator is written over a type that has no instance of the trait it is. |
| `L0418` | A trait method is used at a type with no instance of its trait. |
| `L0419` | A parameter of a method a trait declares states no type. |

`L0308`, `L0309`, `L0310`, and `L0311` are raised by name resolution, which
`crates/resolver/src/error.rs` words.
`L0401`, `L0418`, and `L0419` are raised by type inference, which `crates/types/src/error.rs`
words.

## Properties

These hold and are checked with property-based tests:

1. A trait, an instance, and a constraint each survive printing and parsing unchanged.
2. Canonical form is idempotent over a file holding a trait and an instance.
3. A trait method called at a type with an instance resolves to that instance and to no other.
4. A generic constrained by a trait is written once per type it is used at, and each body calls
   the instance of that type.
5. A program with two instances of one trait for one type is refused, whatever order they are in.
6. `==` is accepted over exactly the types that have an instance of `Eq`.
7. `is_less`, `hashed`, and `shown` are each accepted over exactly the types that have an
   instance of the trait declaring them, as `==` is.

What each instance over those three types answers is a claim about a running program, so it is
held to by `tests/spec/traits/supplied.lm` rather than by a property: that `is_less` is a total
order over `Bool`, `Int`, and `String` and is transitive, that equal values hash alike, and that
`shown` renders each of the three as this spec states.

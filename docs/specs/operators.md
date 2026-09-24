# Operators

## Intent

Every operator is a trait method, and `==` is only the first to be written that way.

`docs/design.md` section 8 names the traits: `+` is `Add`, `-` is `Sub`, `*` is `Mul`, `/` is
`Div`, `%` is `Rem`, prefix `-` is `Neg`, and `<`, `<=`, `>`, and `>=` are `Ord`.
`docs/specs/traits.md` states what a trait and an instance are; this spec states which trait each
operator is, what the library writes, and what a declared type writes to own one.

Version 0.1 wired `Int` and `String` to the operators by name, so no declared type could have one.
That wiring was the degenerate case of this design rather than a design of its own, and it goes:
`Int` gets `+` from an instance exactly as a declared `Money` does.

`&&`, `||`, and `!` stay `Bool`'s alone.
Each of them decides whether to run the other side, and an instance is a call, which cannot.

`total += value` is `Add` as much as `total + value` is, and asks the trait of what is assigned to.
It is the one operator written as a statement, and it is the only one written that way.

## The traits

```text
trait Add<T> {
    fn add(one: T, other: T) -> T
}

trait Sub<T> {
    fn subtract(one: T, other: T) -> T
}

trait Mul<T> {
    fn multiply(one: T, other: T) -> T
}

trait Div<T> {
    fn divide(one: T, other: T) -> Option<T>
}

trait Rem<T> {
    fn remainder(one: T, other: T) -> Option<T>
}

trait Neg<T> {
    fn negate(value: T) -> T
}

trait Ord<T> {
    fn is_less(one: T, other: T) -> Bool
}
```

An operator takes two values of one type and its trait's method says what it gives back.
`Add`, `Sub`, `Mul`, and `Neg` give back the type they were given.
`Div` and `Rem` give back an `Option` of it, because a zero divisor has no answer to give.
`Ord` gives back a `Bool`, because it answers a question about two values rather than building one.

`Ord` declares the one method the four comparisons are written with:

| Written | Called |
| --- | --- |
| `one < other` | `is_less(one, other)` |
| `one > other` | `is_less(other, one)` |
| `one <= other` | `!is_less(other, one)` |
| `one >= other` | `!is_less(one, other)` |

One method is what an instance writes, and the other three spellings follow from it, so no
instance can say that `a < b` and `b > a` disagree.
`Ord` is one of the standard traits `docs/specs/traits.md` declares, and this spec gives the four
comparisons the trait to resolve to; `docs/specs/derive.md` states what a type derives it as.

## What the library writes

`library/prelude.bx` is Bux source the compiler carries, which `docs/specs/library.md` states,
and it declares the seven traits and writes every one of these instances:

```text
instance Add<Int>       instance Add<String>
instance Sub<Int>
instance Mul<Int>
instance Div<Int>
instance Rem<Int>
instance Neg<Int>
instance Ord<Int>      instance Ord<String>      instance Ord<Bool>
```

`Int` has every one of them, and `String` has `Add`, which joins two of them.
`Ord` is the one that reaches past `Int`, because ordering is not arithmetic: `docs/specs/traits.md`
states the order the library writes over `Bool` and over `String`.
Every other operator over a `String` or a `Bool` is `L0406` as it was.

Each trait's name and each method's name is an ordinary prelude name rather than a keyword, so a
module declaring `Add` or `add` is refused with `L0302`, exactly as one declaring `Eq` is.
A module writing `instance Add<Int>` is refused with `L0308`, because there already is one.

None of these instances has a body anything calls.
What one amounts to is written out where the operator is written, as `Eq`'s instances are: two
whole numbers added as the JVM adds them, and two strings joined as `+` already joined them.
The body in `library/prelude.bx` is what says in Bux what that instruction does.

## A declared type owning an operator

```text
instance Add<Money> {
    fn add(one: Money, other: Money) -> Money {
        Money { cents: one.cents + other.cents }
    }
}
```

`one + other` over two `Money`s is the call of the `add` that instance writes.
Nothing else is needed: the operator is a method, and an instance is how a type gets one.

A type with no instance has no operator.
`one + other` over two values of a type that has no `Add` is refused with `L0406`, where it is
written, and the same holds of every other operator and its trait.

An operator is resolved the way every trait method is, which `docs/specs/traits.md` states: by the
type its trait's parameter settled on, while the program is compiled, reaching one instance.
Inside `fn total<T: Add<T>>(items: List<T>)`, `one + other` reaches the constraint `T` declares,
and the body written for `T = Money` calls the `add` of `instance Add<Money>`.

## What a type nothing settled takes

A type that nothing settled is an `Int`, which is the one default the language keeps.
`1 + 1` adds two `Int`s, and so does `one + other` in a function that says no more about either.
The default is taken before the trait is asked, so what is asked for is `Add<Int>`.

## What is written

An operator over a type whose instance a module wrote is an `invokestatic` of that instance's
method, named as `docs/specs/traits.md` names one: `Add$Money$add`.

An operator over one of the types the JVM holds is the instruction it always was:
`ladd` for `Int`, a concatenation for `String`, and a compared pair of longs for `Ord<Int>`.
`Ord<Bool>` and `Ord<String>` are likewise written out where they are written, as
`docs/specs/traits.md` states what each one answers.
`/` and `%` keep the zero-divisor test `docs/specs/codegen.md` states, because `ldiv` throws and no
method a module writes may throw.

`<=`, `>`, and `>=` over an instance a module wrote are the one call `is_less` names, with its
arguments the way round the table above gives and the answer flipped where the table flips it.

`total += value` writes what `total = total + value` writes, except that what the name holds is
read once: it is loaded, and the two are the `add` arguments in the order the source writes them.

## The errors

| Code | Raised when |
| --- | --- |
| `L0302` | A module declares one of the seven traits, or one of their methods. |
| `L0308` | A module writes an instance the prelude already has. |
| `L0400` | The two sides of an operator are values of two different types. |
| `L0406` | An operator, `+=` among them, is written over a type with no instance of its trait. |
| `L0407` | A division is written with a `0` the compiler can already see. |

`L0406` was `==` alone, and now every operator is refused with it, because every operator asks the
same question: does this type have the trait this operator is?

## Properties

These hold and are checked by drawn properties, each a test of `tests/`:

1. Each operator over `Int` gives the type and the answer it gave before it was a trait.
2. An operator over a type with an instance calls that instance and no other.
3. An operator over a type with no instance of its trait is refused, whatever the operands are.
4. Each of the four comparisons over a type with an instance of `Ord` is one call of that
   instance's `is_less`, whichever way round it is written.

That the four agree at run time — `one < other` and `other > one` answering alike — is what the
examples under `tests/spec/operators/` run, because agreeing is about answers rather than about
the instructions the four are written out as.

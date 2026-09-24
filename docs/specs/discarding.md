# Discarding a value

A value a program works out and then throws away was thrown away on purpose or by mistake, and in
most languages the two are spelled the same.
Bux spells them differently: a value is discarded where the source writes `_ =`, and nowhere else.

## Intent

`docs/design.md` section 5 says no program throws and no operation is partial, so a failure is a
value rather than an event: a `Result` the caller is holding.
A held value that nothing reads is a swallowed failure, which is the one way a language without
exceptions can still lose one.

mycs lints `Swallowed Exception` after the fact.
The language removes the case instead, the way it removes a null: the program does not compile.

## The rule

A statement written for its effect has nothing to leave behind, so its type is `()`.

```text
fn main(arguments: List<String>) -> Int {
    save(user)
    0
}
```

`save` gives back a `Result<(), Error>`, and nothing here takes it, so this does not compile.

A statement's value is discarded when nothing takes it.
The last statement of a block gives the block its value, and every statement above it is discarded.
A function body gives its value to the result the function declares.
An `if` or a `match` written as an expression gives its value to the expression it is written in,
so a branch that ends in a `Result` is refused where the whole expression is, once, rather than
once per branch.
A `for` body gives its value to nothing at all, so its last statement is discarded like the rest.

A statement whose type nothing settles is `()`, as an unsettled addition is `Int`.
`todo("not yet")` written as a statement is therefore accepted: a hole takes the type expected of
it, and `()` is what is expected there.

## Saying it on purpose

`_ = save(user)` discards the value and says so.

```text
fn main(arguments: List<String>) -> Int {
    _ = save(user)
    0
}
```

The value is worked out and dropped, and the line says that is what was meant.
A reader looking for what a function ignores greps for `_ =` and finds every one.

`_` is not a name.
As a statement, it is written on the left of a single `=` and nowhere else.
So `_`, `let _ = 1`, `var _ = 1`, and `_ += 1` are each refused by the grammar.
Nothing reads `_` back, because `_ =` binds nothing.
The same `_` is a pattern, which `docs/specs/patterns.md` states.
It is also the binding of a `for` that reads none of its values, as in `for _ in users`.
There it binds nothing too, and `docs/specs/grammar.md` states it.

## The error

A discarded value is `L0408`, raised by inference and worded by `src/refusal.bx`:

```text
error[L0408]: `Result<(), Error>` is left here and nothing takes it
  --> demo.bx:2:5

  2 |     save(user)
    |     ^^^^^^^^^^

help: write `_ = ` in front of it to throw the value away on purpose
```

The message names the type that is left, because that is what tells the reader whether dropping it
is a mistake: a `Result` almost always is, and something else may not be.

## Properties

These hold and are checked by drawn properties, each a test of `tests/`:

1. A statement of type `()` is accepted wherever it is written in a block.
2. A statement of any other type is accepted as a block's last statement and refused above it.
3. `_ = ` in front of a statement makes it compile, whatever type the statement has.

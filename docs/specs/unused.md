# A name that nothing reads

Every name that a program introduces is read at least once, or the program does not compile.
This spec states the rule for a binding and for a parameter.
`docs/specs/modules.md` states the same rule for an import, which is `L0320`.

## Intent

A name that nothing reads is a mistake, or it is a leftover of a change.
A reader who finds the name looks for its use, and there is none.
`docs/specs/discarding.md` refuses a value that nothing takes.
This rule refuses a name that nothing reads, for the same reason.

## The rule

A name is read where the program uses it as a value.
An assignment is not a read, and `+=` is not a read, so a `var` that is only assigned is refused.

These names are each held to the rule, and each is `L0321` where nothing reads it:

- a `let` binding
- a `var` binding
- a `for … in` binding
- a name that a pattern binds, in a `match` arm
- a parameter of a function that chooses its signature

```text
fn shipped(order: Order) -> Int {
    let weight = order.items * 2
    order.items
}
```

Nothing reads `weight`, so this is refused at `weight`:

```text
error[L0321]: the binding `weight` is read by nothing
help: read it, or write `_ =` for its value
```

A parameter that nothing reads has the message ``the parameter `x` is read by nothing``.
Its help is `read it, or remove it`, because each call then drops the argument too.
The span is the name where the program introduces it.

## What to write instead

`_ =` discards a value on purpose, as `docs/specs/discarding.md` states.
`_` is a pattern that binds nothing, so `Some(_)` matches a value that the arm does not need.
`for _ in values` runs the body once for each element and binds no name.
`Box {}` matches a record and binds no field, as `docs/specs/patterns.md` states.

## The exemption

The rule does not hold a parameter of a signature that the function does not choose:

- the `arguments` of `main`
- each parameter of an instance method
- the parameters of `start` and of `receive` in a process

The signature of `main` is fixed, and 281 of 294 `main` functions read no arguments.
A trait fixes the signature of an instance method, and a process fixes `start` and `receive`.
A binding inside those functions is held to the rule all the same.

## A hole reads every name in scope

A hole stands for code that is not written yet, and that code can read any name in scope.
So a hole reads every name in scope where it is written, and `bux check` accepts this:

```text
fn shipped(order: Order) -> Int {
    let weight = order.items * 2
    todo("price the parcel")
}
```

A name below the hole is not in scope at the hole, so the hole does not read it.

## What holds it

`src/resolver.bx` refuses the name after it resolves the module.
`tests/unread.bx` holds two drawn properties:

- A module with one unread name is refused, and the refusal names the kind and the name.
- The same module, with the name read, is accepted.

The drawn shapes are an import, an import that only an example reads, and a `let` and a `var`.
The others are a `for` binding, a pattern binding, a parameter, and a binding below a hole.

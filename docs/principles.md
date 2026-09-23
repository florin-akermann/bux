# Bux: Design Principles

How features are judged for `docs/design.md`; the questions come before any feature does.

## 1. Design principles

When deciding whether to add a feature, ask:

### 1. Does it make everyday code simpler?

If not, it needs a very strong justification.

### 2. Can the compiler guarantee something useful?

Prefer language features that eliminate classes of bugs.

### 3. Does it make domain concepts explicit?

Prefer:

```text
UserId
Money
Email
PolicyId
```

over:

```text
String
Int
Decimal
```

everywhere.

### 4. Can the feature be explained simply?

A feature that requires several pages of rules is suspicious.

### 5. Does it belong in the language?

Prefer libraries over language features where possible.

### 6. Does the JVM implementation leak into the language?

If yes, reconsider the abstraction.
`docs/design.md` section 2 states the rule this asks against: the JVM is a target, not a model.
Identity, `equals` on everything, boxing a program can see, and a special `int` are all leaks.

### 7. Can the mistake be made unwriteable?

A feature that catches a mistake is weighed against a shape that leaves the mistake unwriteable.
A linter reports a swappable pair of arguments after the fact, and the call still compiles.
`docs/design.md` section 11 states what the language does instead.
Where order alone holds two arguments apart, the call names them, and a swapped pair is refused.
Elegance leaves the invalid case unwriteable; adequacy writes it and rejects it afterwards.

### 8. Could a library declare it?

A type the prelude supplies must be a type a library could have declared instead.
A capability the language gives `Int` and withholds from a declared type is a leak of its own kind.
A literal, an operator, and `Eq` are the three such capabilities, and each is a trait method.
`docs/design.md` sections 3 and 8 state how they reach every type alike.
A feature that gives a prelude type a fourth is refused until a declared type can have it too.

### 9. Is it a second way to write something the language already writes?

Syntactic sugar is a second spelling, and a second spelling is a cost with no guarantee behind it.
A reader learns both, a formatter chooses between them, and every later feature answers to two
forms rather than one.
`++`, `--`, `-=`, `*=`, `/=`, `%=`, a ternary `?:`, and a compound assignment of any other operator
are all refused: each one writes what `a = a + 1` and an `if` already write plainly.
`+=` is the one shorthand version 0.1 kept, because a `for` loop that totals is the everyday shape
Bux is built around, and it is the ceiling rather than the first of a set.
A new shorthand lands only where it removes a class of mistake, never where it only removes typing.

### 10. Is anything exempt from it?

A rule that holds for every type, function, and operator but one is a special case, and is refused.
A prelude type is a type a library could have declared, and question 8 holds it to that.
An operator is a function with other syntax, and `docs/design.md` section 5 gives it no exemption.
`main` is a function like any other: it declares what it gives back, and that is `()`.
A feature that needs a name, type, or operator treated apart from the rest is reshaped or refused.

A rule also states the scope it holds over, because an unstated scope reads as a rule it is not.
The no-identity rule is the one that needs saying: every type a program declares is a value.
A process, a scoped resource, and a foreign reference have identity.
A `type` declaration writes none of the three.
They stand outside the rule rather than exempt from it, so the rule itself keeps no exception.
`docs/design.md` sections 10, 14, 15, and 17 state the rule and all three of the things outside it.

### 11. Does it have an answer for every input?

An operation with no answer for some of its input says so in its type, never at runtime.
Rust panics on `x / 0` and calls the panic a design; Bux does not, so `17 / 0` is `None`.
There is no panic, no exception, no `unwrap`, and no runtime failure a program can reach.
A feature that would crash on some input is refused until its type carries that case instead.
`docs/design.md` section 5 states the rule, and `docs/specs/arithmetic.md` works it through.

### 12. Could a `for` loop write it instead?

The standard library is small, and the fewer methods a type has, the better.
A method lands only where a plain loop over what the type already exposes cannot write it.
A map has no iterator, and no type has a `for_each`: the `for` loop is what Bux is built around.
A method that only saves the reader a loop is refused, the same as sugar under question 9.
`docs/implementation.md` section 4 states the library's scope.

---

## 2. The central trade-off

The language deliberately chooses:

```text
Rust:
    memory safety
    zero-cost abstractions
    ownership
```

in favor of:

```text
JVM:
    GC
    mature runtime
    ecosystem
    JIT
```

while retaining:

```text
Rust/Haskell:
    strong static types
    ADTs
    exhaustive matching
    inference
```

and:

```text
Go:
    simplicity
    tooling
    pragmatic deployment
```

and:

```text
Erlang:
    a process owns its state
    a message is the only way to that state
    a process is cheap
```

The goal is not to maximize language power.

The goal is:

> Maximize useful guarantees per unit of language complexity.

---

## 3. One-sentence description

> A small ML-inspired language with Go-like syntax and tooling, compiled to the JVM by a Bux compiler.

Or more succinctly:

> Go's simplicity.
> Haskell's types.
> Erlang's messages.
> The JVM's runtime.
> A compiler written in itself.

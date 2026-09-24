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
Elegance leaves the invalid case unwriteable; adequacy writes it and rejects it afterwards.

### 8. Could a library declare it?

A type the prelude supplies must be a type a library could have declared instead.
A capability the language gives `Int` and withholds from a declared type is a leak of its own kind.
`docs/design.md` sections 3 and 8 name the three such capabilities and how each reaches every type.
A feature that gives a prelude type a fourth is refused until a declared type can have it too.

### 9. Is it a second way to write something the language already writes?

Syntactic sugar is a second spelling, and a second spelling is a cost with no guarantee behind it.
`docs/design.md` section 2 states the sugar rule and the test a new shorthand has to pass.
`docs/rationale.md` section 2 gives the reasons.

### 10. Is anything exempt from it?

A rule that holds for every type, function, and operator but one is a special case, and is refused.
`docs/design.md` section 2 states the rule for a prelude type, an operator, and `main`.
A feature that needs a name, type, or operator treated apart from the rest is reshaped or refused.

A rule also states the scope it holds over, because an unstated scope reads as a rule it is not.
The no-identity rule is the example: `docs/design.md` section 10 states it and its scope.
The three things with identity stand outside the rule rather than exempt from it.

### 11. Does it have an answer for every input?

A feature that would crash on some input is refused until its type carries that case instead.
`docs/design.md` section 5 states the rule, and `docs/specs/arithmetic.md` works it through.

### 12. Could a `for` loop write it instead?

The standard library is small, and the fewer functions a module offers for a type, the better.
A function lands only where a plain loop over what the type already exposes cannot write it.
A map has no iterator, and no type has a `for_each`: the `for` loop is what Bux is built around.
A function that only saves the reader a loop is refused, the same as sugar under question 9.
A higher-order function such as `map` is a loop written twice: the loop, and a function to pass.
A method is a trait method only; `list.push` is a function of the module `list`, not a method.
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

> A small ML-inspired language with Go's philosophy and tooling, compiled to the JVM by a Bux compiler.

Or more succinctly:

> Go's simplicity.
> Haskell's types.
> Erlang's messages.
> The JVM's runtime.
> A compiler written in itself.

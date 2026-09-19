# Lumen: Design Principles

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
    concurrency
    pragmatic deployment
```

The goal is not to maximize language power.

The goal is:

> Maximize useful guarantees per unit of language complexity.

---

## 3. One-sentence description

> A small ML-inspired language with Go-like syntax and tooling, compiled to the JVM by a Rust compiler.

Or more succinctly:

> Go's simplicity.
> Haskell's types.
> The JVM's runtime.
> A Rust compiler.

# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🟡 Item 044: A type a module declares is reachable from the module that imports it
**Depends on:** Item 040 — version 0.1 closes before the first 0.2 item opens.
`docs/specs/modules.md` keeps a type its declaring module's own: `demo.User` cannot be written.
A function whose signature names one is refused where it is reached, as `L0416`.
A library written in Lumen is impossible until an importing module can name the types it offers.
A type is reached the way a function is, through the module's name: `demo.User`, `demo.Payment`.
Its variants are reached the same way, so a `match` over `demo.Payment` names `demo.Pending`.
[044][a] - `docs/design.md` section 16 states the rule; `docs/specs/modules.md` states the scopes.
[044][b] - Parsing a dotted type and a dotted pattern, test-first; canonical form settles spacing.
[044][c] - Resolving and typing across the module boundary, test-first; `L0416` is retired.
[044][d] - Executable examples under `tests/spec/modules/`: a record, an ADT, a `match` across.

## 🔴 Item 045: The prelude and the standard library are Lumen source
**Depends on:** Item 033, Item 044 — the operator instances need a home; a library needs names.
`crates/resolver/src/prelude.rs` says the prelude becomes Lumen source once a module can be loaded.
A module loads since Item 039, so the prelude's types, constructors, and `or` move to `.lm`.
The `Int` and `String` instances of the operator traits, and of `Eq`, live in the same source.
`List` gains its everyday functions in Lumen: `length`, `push`, `contains`, `map`, and `filter`.
`String` gains `length`, `contains`, `split`, and `join`, written against what `+` already gives.
The compiler finds the library beside itself, so no build and no test needs the network.
[045][a] - Spec first in `docs/specs/library.md`: where the source is, how it is found, its parts.
[045][b] - Loading treats the library as modules, test-first; the prelude is loaded unasked.
[045][c] - `prelude.rs` and the checker's built-in instances are removed; every test still passes.
[045][d] - `List` and `String` functions, each with an executable example in `tests/spec/library/`.

## 🔴 Item 046: `Map<K, V>` and `Set<T>` are library types
**Depends on:** Item 043, Item 045 — a key needs `Eq` and `Hash`; a library type needs a library.
`docs/implementation.md` section 4 names collections; version 0.1 has only `List`.
`Map<K, V>` and `Set<T>` are declared in Lumen, and `K: Hash<K>` says what a key must be.
Both are values: two maps holding the same entries are one value, and neither has identity.
There is no literal for either yet; a map is built by `empty` and `insert`, and read by `get`.
`get` gives an `Option<V>`, because a key that is absent is a case the type has to say.
[046][a] - `docs/design.md` section 9 states the two types and what a key must have.
[046][b] - Spec first in `docs/specs/collections.md`: each function, its type, and its cost.
[046][c] - Both types in Lumen, test-first; a property: `get` after `insert` gives what went in.
[046][d] - Executable examples under `tests/spec/library/`, including a key without `Hash`.

## 🔴 Item 047: Richer patterns: `_`, a literal, and an or-pattern
**Depends on:** Item 040 — version 0.1 closes before the first 0.2 item opens.
**Depends on:** Item 034 — a literal pattern over a declared type needs the trait a literal is.
A pattern today is a bare name or a constructor with patterns inside; nothing else is written.
`docs/implementation.md` section 10 promises richer pattern matching and does not say richer how.
Three forms earn their keep in everyday code: `_`, a literal, and an or-pattern.
`_` ignores a value, `0` or `"quit"` matches one, and `A | B` answers two variants in one arm.
A guard is not among them: an `if` inside the arm reads the same and keeps exhaustiveness simple.
Exhaustiveness extends to each form; a literal pattern needs a `_` or a binding arm after it.
A whole number written as a pattern is an `Int` today, which `docs/specs/literals.md` states.
So `match count { 5 => … }` over an `Int32` is `L0400`, though `count + 5` is accepted, and a
pattern needs both traits: `IntegerLiteral` to become the type, and `Eq` to say what sameness is.
[047][a] - `docs/design.md` section 4 states the three forms and refuses the guard, with the reason.
[047][b] - Spec first in `docs/specs/patterns.md`: each form, its canonical spacing, its errors.
[047][c] - Parsing and exhaustiveness, test-first; a property: the check agrees with enumeration.
[047][d] - A whole-number pattern takes the type it is matched against and compares by its `Eq`.
[047][e] - Executable examples under `tests/spec/patterns/`, including a literal match with a gap.

## 🔴 Item 048: A Java class is reached through an `extern` declaration
**Depends on:** Item 044, Item 045 — a wrapper is a library module offering its own types.
`docs/implementation.md` section 3 sketches `extern java class` and says nothing of the boundary.
`docs/design.md` section 2 says what never crosses: identity, `null`, exceptions, the object model.
An `extern` declaration names a static method or a constructor and gives it a Lumen signature.
A `null` given back becomes `None`, a thrown exception becomes `Err`, and nothing else shows.
A Java object a Lumen program holds is a value it cannot compare, hash, or print until a trait says.
`io` and `files` become Lumen modules over `extern` declarations; the compiler stops supplying them.
[048][a] - `docs/design.md` gains a section: what an `extern` reaches, and what never crosses.
[048][b] - Spec first in `docs/specs/interop.md`: the declaration, the two mappings, and the errors.
[048][c] - Lowering emits the call and the two mappings, test-first, asserted on the instructions.
[048][d] - `io` and `files` in Lumen; `docs/specs/io.md` loses the "supplied module" paragraph.
[048][e] - Executable examples under `tests/spec/interop/`, skipped by name when there is no JDK.

## 🔴 Item 049: A package is a directory of modules with a name and a version
**Depends on:** Item 044, Item 045 — a package offers types; the library is the first package.
`docs/implementation.md` section 10 names package management; no document says what a package is.
Version 0.1 finds a module beside the importing file and nowhere else.
A package is a directory with a manifest naming it and its version, and its modules are its files.
A dependency is a package in a named directory, and the manifest lists it; nothing is fetched.
Fetching, a registry, and a lockfile wait for a concrete requirement, which offline-first defers.
[049][a] - `docs/design.md` section 16 states what a package is and how an import reaches one.
[049][b] - Spec first in `docs/specs/packages.md`: the manifest, the directory, and the errors.
[049][c] - Loading reaches a dependency's module, test-first; a ring across packages is `L0307`.
[049][d] - Help topic in `crates/cli/src/help/`; `lumen check` and `lumen build` take a package.
[049][e] - Executable examples under `tests/spec/packages/`: two packages, one importing the other.

## 🔴 Item 050: A call is written with its first argument in front
**Depends on:** Item 041 — the resolver and the checker are mid-change until it lands.
`docs/design.md` section 11 makes `maybe.or(fallback)` the call `or(maybe, fallback)`.
The parser already reads `maybe.or(0)` as a call of the field `or`, and the checker refuses it.
`Option<Int>` has no field named `or`, so `L0402` is what the form gets today.
The name before the dot decides: a module is reached into, and a binding is passed first.
Which function is called is settled by the name alone, so no type is looked up to find it.
The receiver counts as an unnamed argument, so a call that must name them keeps its plain form.
[050][a] - Spec first in `docs/specs/calls.md`: the form, the three dots, naming, and the errors.
[050][b] - The resolver looks the callee up in scope when the receiver is no module, test-first.
[050][c] - The checker and lowering treat it as the plain call; a property: the two forms agree.
[050][d] - The formatter keeps the form, and a round trip covers it.
[050][e] - Executable examples under `tests/spec/calls/`; the `or` examples in `docs/specs` move.


## 🔴 Item 051: Identity is quarantined, not abolished
`docs/design.md` section 15 argues that no two spawned functions ever hold the same value.
The section's own example refutes it: `events` is held by the parent and by the spawned `produce`.
A channel is identity-bearing, because two holders of one channel is the whole point of a channel.
`send` mutates a queue and `receive` observes the mutation, which is what makes the example work.
The rule that survives the example is narrower and stronger than the one the section states.
A channel and a scoped resource are the only identity-bearing things, and neither is declarable.
Every type a program declares is a value, so a program has nothing to build a lock out of.
A channel is written by the compiler and the runtime, which is why it is the one safe exception.
[051][a] - `docs/design.md` section 15 states the narrower rule and drops the overreaching one.
[051][b] - Sections 10 and 14 take that wording, so the three say one thing about identity.
[051][c] - `docs/principles.md` line 10 gains the scope it assumes: a declared type has no identity.

## 🔴 Item 052: A foreign reference is not sendable
`docs/design.md` section 14 refuses a resource-typed value returned, stored in a field, or sent.
That "or sent" clause is the whole of the concurrency rule, and section 15 leans on it unsaid.
A Java object has identity and mutation, so one sent on a channel is shared mutable state again.
Every clause section 15 argues from fails the moment a foreign reference crosses a spawn.
The answer is the rule already written: a foreign reference escapes exactly as a resource does.
That is no new mechanism, and it is why the escape check is one rule rather than a family of them.
Item 048 lands the declaration that makes such a reference writable, so this item comes before it.
[052][a] - `docs/design.md` section 14 names a foreign reference among what the check refuses.
[052][b] - Section 15 cites the clause where it argues nothing is shared, so the two sections agree.
[052][c] - Item 048's `docs/specs/interop.md` points at the clause rather than restating the rule.

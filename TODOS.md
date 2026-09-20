# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🟢 Item 034: A literal takes the type its context expects
**Depends on:** Item 033 — a literal is a trait method, and the operator traits land first.
`1` is an `Int` today, at the one place inference types a literal, so `Int32` is written `Int32(1)`.
`docs/design.md` section 8 gives a whole-number literal the type its context expects instead.
That needs the type to have `IntegerLiteral`; a literal that nothing settles is still an `Int`.
A literal that does not fit its type is a compile error where it is written, never a wrapped value.
`let x: Int32 = 5_000_000_000` is refused; how an instance states what fits is the spec's question.
[034][a] - Spec first in `docs/specs/literals.md`: the trait, its bounds, and the error text.
[034][b] - Inference gives a literal a variable constrained by `IntegerLiteral`, defaulted to `Int`.
[034][c] - The fit check at compile time, test-first; the `Int` instance accepts every literal.
[034][d] - Executable examples under `tests/spec/literals/`: `Int32` as a literal, and a misfit.

## 🔴 Item 042: A record or a variant derives `Eq`
**Depends on:** Item 041 — a derived instance is an instance.
`docs/design.md` section 8 promises `derive Eq` for 0.2, and refuses `==` on a record until then.
Equality is by state, so the compiler can write the instance: field by field, variant by variant.
A record derives `Eq` only when every field's type has `Eq`, and the refusal names the field.
The instance the compiler writes is the one the author would have, so nothing about `==` changes.
[042][a] - Spec first in `docs/specs/derive.md`: the form, what is written, the missing-`Eq` error.
[042][b] - Lowering writes the instance, test-first; a property: derived `==` agrees with state.
[042][c] - `L0406` gains a `help:` that names the `derive` to write.
[042][d] - Executable examples under `tests/spec/traits/`: a derived record, a field without `Eq`.

## 🔴 Item 043: `Ord`, `Hash`, and `Show` are standard traits, and derivable
**Depends on:** Item 042 — the second derivable trait follows the first's path.
`docs/design.md` section 8 names `Eq`, `Ord`, `Hash`, and `Show` as the standard traits.
Each is a declared trait the library ships, with instances for `Int`, `Bool`, and `String`.
`Hash` is a trait a type opts into, not the JVM's `hashCode`; nothing has a hash it did not ask for.
`Show` renders a value as text a reader can read; nothing has a `toString` it did not ask for.
A record derives each field by field in declaration order; a variant by variant, then by payload.
Item 033 makes `<` and its kin resolve through `Ord`; this item gives them the trait to resolve to.
[043][a] - Spec first in `docs/specs/traits.md`: each trait's method, its laws, what derive writes.
[043][b] - The three traits and the prelude's instances, test-first; properties cover each law.
[043][c] - Derive for each, test-first; a property: derived `Ord` is total and agrees with `Eq`.
[043][d] - Executable examples under `tests/spec/traits/`, one file per trait.

## 🔴 Item 044: A type a module declares is reachable from the module that imports it
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
A pattern today is a bare name or a constructor with patterns inside; nothing else is written.
`docs/implementation.md` section 10 promises richer pattern matching and does not say richer how.
Three forms earn their keep in everyday code: `_`, a literal, and an or-pattern.
`_` ignores a value, `0` or `"quit"` matches one, and `A | B` answers two variants in one arm.
A guard is not among them: an `if` inside the arm reads the same and keeps exhaustiveness simple.
Exhaustiveness extends to each form; a literal pattern needs a `_` or a binding arm after it.
[047][a] - `docs/design.md` section 4 states the three forms and refuses the guard, with the reason.
[047][b] - Spec first in `docs/specs/patterns.md`: each form, its canonical spacing, its errors.
[047][c] - Parsing and exhaustiveness, test-first; a property: the check agrees with enumeration.
[047][d] - Executable examples under `tests/spec/patterns/`, including a literal match with a gap.

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

## 🔴 Item 051: A whole number matches a declared type
**Depends on:** Item 034 — a literal is `IntegerLiteral`, and matching one needs `Eq` as well.
`docs/specs/literals.md` states the limit: a whole number written as a pattern is an `Int`.
`match count { 5 => … }` over an `Int32` is therefore `L0400`, though `count + 5` is accepted.
A pattern asks whether two values are the same, which is `Eq` rather than `IntegerLiteral`.
So a pattern needs both: the number becomes the type, and the type says what sameness is.
Exhaustiveness is the second question: a range of whole numbers is never listed arm by arm.
[051][a] - Spec first in `docs/specs/literals.md`: what a pattern asks, and what it still needs.
[051][b] - The pattern takes the type it is matched against, test-first, and is held to its bounds.
[051][c] - Lowering compares through the type's `Eq`, and exhaustiveness keeps its wildcard rule.
[051][d] - Executable examples under `tests/spec/literals/`: a match that answers, and one refused.

# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 107: A foreign reference cannot escape the function that reaches it
`docs/design.md` section 14 states the escape check, and Item 101 found that it is not built.
A function can still return a value of an `extern` type, and a type can still hold one in a field.
`L0811` guards a process only, so the rule holds for a process and not for the rest of a program.
[107][a] - `docs/specs/interop.md` states where a foreign reference can and cannot go.
[107][b] - A diagnostic refuses a return, a field, or a constructor argument of an `extern` type.
[107][c] - A `tests/spec/interop/` example shows each refused place, and a drawn property holds it.

## 🔴 Item 108: A top-level function writes its signature
`docs/design.md` section 6 asks for a signature "when useful".
`bux api` prints `_` where nothing settled a type, so a reader of the page opens the file.
An inference error then surfaces at a distant call instead of inside the body that caused it.
A written signature is a contract at the boundary, and inference keeps its work inside a body.
[108][a] - `docs/design.md` section 6 and `docs/specs/types.md` state the rule.
A top-level function writes every parameter type and its result type; a nested one stays inferred.
[108][b] - A diagnostic refuses a top-level function whose signature omits a type.
Its `help:` line spells the inferred type where inference settled one.
[108][c] - `docs/specs/api-surface.md` drops the `_` case, because no top-level function has one.
[108][d] - `tests/spec/type_inference/` shows the refusal.
Every `.bx` under `compiler/` and `tests/` writes its signatures.

## 🔴 Item 109: An unused import, binding, or parameter is refused
No spec states what happens to a name that nothing reads.
An edit that replaces a body leaves the imports and bindings the old body used.
Nothing reports them.
A parameter a body never reads is a signature that says more than the function does.
`_ =` already spells a discard, so the rule adds no spelling, only a refusal.
[109][a] - `docs/specs/modules.md` refuses an import that no declaration of the module reaches.
[109][b] - A new spec refuses a `let`, a `var`, a `for` binding, or a pattern binding nothing reads.
It refuses a parameter the body never reads, and says how a `for` over a count writes its binding.
[109][c] - The spec settles a parameter an instance method declares but does not read.
It settles a `main` that reads no arguments the same way.
[109][d] - `tests/spec/name_resolution/` shows each refusal.
A drawn property holds that every name the compiler accepts is read at least once.

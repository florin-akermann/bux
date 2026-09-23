# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 078: A positional call to a function declared above its caller compiles
**Depends on:** nothing — the defect is in the Rust type checker as it is.
A positional call in mutual recursion panics at `crates/types/src/infer/arguments.rs` (`takes`).
The `unreachable!` says a function has its type before anything below it calls it, which is false.
Items 067 to 070 wrote named arguments at every such call, and Item 071 would write many more.
[078][a] - A spec example calls two functions positionally in mutual recursion, and it compiles.
[078][b] - The positional-argument rule reads the declared signature however the module is walked.
[078][c] - No `unreachable!` or `expect` in `crates/types` rests on the order a body is walked in.

## 🔴 Item 079: A match that gives variants of another module's type passes JVM verification
**Depends on:** nothing — the defect is in the Rust lowering as it is.
A `match` whose arms give `other.Held(1)` and `other.Free` fails as `VerifyError: Bad return type`.
At the join the stack holds `Object`, where the method returns the sum type of the other module.
Items 067 to 070 wrote early returns or arms of one record type to avoid it.
[079][a] - An expect-run example under `tests/spec/modules` gives both variants from a `match`.
[079][b] - The join of a `match`, an `if`, and a block carries the sum type across modules.

## 🔴 Item 071: Type inference is written in Bux
**Depends on:** Item 064, Item 065, Item 070, Item 078, Item 079 — unification keys a table.
This is the largest phase, at 5,313 lines of Rust, and mutable tables become returned values.
[071][a] - `compiler/types.bx` infers, unifies, resolves constraints, and derives per the specs.
[071][b] - The harness compares every diagnostic and every `api` surface with the Rust phase's.

## 🔴 Item 072: Exhaustiveness and holes are checked in Bux
**Depends on:** Item 071 — both read the typed tree.
[072][a] - `compiler/exhaustiveness.bx` reports every gap `docs/specs/exhaustiveness.md` names.
[072][b] - `compiler/holes.bx` lists every `todo` as `docs/specs/holes.md` states.

## 🔴 Item 073: Lowering and the class-file writer are written in Bux
**Depends on:** Item 061, Item 071 — the writer puts bytes on disk, and lowering reads types.
The writer has 172 sites of narrow integers, and a `bytes` module hides `% 256` behind names.
[073][a] - `compiler/ir.bx` lowers the typed tree to the JVM IR `docs/specs/codegen.md` states.
[073][b] - `compiler/jvm.bx` writes a class file, with a `bytes` module for `u1`, `u2`, and `u4`.
[073][c] - The harness holds every class file byte for byte equal to the Rust writer's output.

## 🔴 Item 074: The `bux` command line is written in Bux
**Depends on:** Item 060, Item 062, Item 068, Item 072, Item 073 — every command is a phase.
[074][a] - `compiler/main.bx` parses the arguments and runs every command `bux --help` lists.
[074][b] - A `bux` launcher script starts the JVM with `--enable-preview` on the compiled compiler.
[074][c] - Every executable example under `tests/spec` passes under the launcher.

## 🔴 Item 075: The compiler compiles itself
**Depends on:** Item 074 — the fixpoint needs the whole compiler.
[075][a] - Stage 1, built by the Rust `bux`, builds stage 2 from the same source.
[075][b] - A harness holds stage 2 equal to stage 1 byte for byte.
[075][c] - The Rust crates are deleted, and `docs/implementation.md` section 6 says what remains.

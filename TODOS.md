# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🟡 Item 071: Type inference is written in Bux
**Depends on:** Item 064, Item 065, Item 070 — unification keys a table.
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

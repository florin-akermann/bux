# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🟡 Item 066: The lexer is written in Bux
**Depends on:** Item 059, Item 076 — a lexer builds a token list from the code units of a string.
Self-hosting starts with the smallest phase, and the lexer is 286 lines of Rust.
The Rust `bux` compiles the Bux lexer, and `tests/spec/lexer` holds both to one answer.
[066][a] - `compiler/lexer.bx` lexes a module into the tokens `docs/specs/lexer.md` states.
[066][b] - A harness runs the Bux lexer over every `tests/spec/lexer` fixture and compares tokens.
[066][c] - `docs/implementation.md` section 6 names `compiler/` as where the Bux compiler lives.

## 🔴 Item 067: The parser and the AST are written in Bux
**Depends on:** Item 066, Item 077 — the parser reads the tokens the Bux lexer gives, by kind.
[067][a] - `compiler/ast.bx` declares the tree `docs/specs/grammar.md` describes.
[067][b] - `compiler/parser.bx` parses tokens into it, with every `docs/specs/parser` error.
[067][c] - The harness compares the printed tree with the Rust parser's on every parser fixture.

## 🔴 Item 068: The formatter is written in Bux
**Depends on:** Item 067 — the printer reads the source and the tree.
[068][a] - `compiler/format.bx` writes canonical form as `docs/specs/formatting.md` states.
[068][b] - The harness holds it to `format(source) == source` on every `.lm` file in the repository.

## 🔴 Item 069: Modules, packages, and the library source are read in Bux
**Depends on:** Item 061, Item 067 — a loader reads files beside the module and the manifest.
The Rust compiler carries `library/*.lm` with `include_str!`, and Bux has no such thing.
[069][a] - `docs/specs/library.md` states how the Bux compiler carries the library source.
[069][b] - `compiler/modules.bx` loads a module, its imports, and its package as the specs state.
[069][c] - The harness compares the load order and every error with the Rust loader's.

## 🔴 Item 070: Name resolution is written in Bux
**Depends on:** Item 063, Item 069 — scopes are maps keyed by names the program declares.
[070][a] - `compiler/resolver.bx` produces the resolved tree with every name resolution error.
[070][b] - The harness compares its diagnostics with the Rust resolver's on every fixture.

## 🔴 Item 071: Type inference is written in Bux
**Depends on:** Item 064, Item 065, Item 070 — unification keys a table by type variables.
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

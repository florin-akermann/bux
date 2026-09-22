# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 059: A string is read one code unit at a time
No function reads a character of a string, so a lexer cannot be written in Bux.
`String.charAt` and `String.substring` each take an `int`, and `extern` widens a result only.
The spec settles how an `Int` argument reaches an `int` parameter without a partial operation.
[059][a] - `docs/specs/interop.md` states how an `Int` argument crosses to an `int` parameter.
[059][b] - `strings.at(text, index)` gives the code unit as an `Int`, or `None` past the end.
[059][c] - `strings.cut(text, from, to)` gives the substring, or `None` where it does not fit.
[059][d] - An executable example under `tests/spec/library` walks a string and counts its spaces.

## 🔴 Item 060: A program takes arguments, exits with a code, and writes to standard error
`main` takes nothing and gives back nothing, and `io` writes to standard output only.
`docs/specs/diagnostics.md` demands exit codes `0`, `1`, and `2`, and errors on standard error.
A `bux` written in Bux cannot be a command line tool without the three.
[060][a] - `docs/design.md` section 11 states what `main` takes and what a run's exit code is.
[060][b] - `docs/specs/run.md` states how `bux run` passes the arguments and reads the code.
[060][c] - `io.eprintln` writes a line to standard error.
[060][d] - Executable examples under `tests/spec/running` show each of the three.

## 🔴 Item 061: The file system is listed, made, and written
`files` reads one file whole, and a compiler needs the rest of what `crates/cli` reaches.
That is a directory listed, made, and deleted, a file written, and a variable read.
[061][a] - `docs/specs/io.md` states each function, its type, and the `Result` each gives back.
[061][b] - `files.write(path, text)` writes a file whole.
[061][c] - `files.listed(path)` gives the names in a directory.
[061][d] - `files.made(path)` makes a directory, and `files.removed(path)` deletes one.
[061][e] - `environment.read(name)` gives `Some` of a variable and `None` where it is unset.
[061][f] - Executable examples under `tests/spec/io` show each function.

## 🔴 Item 063: A library generic is used at a type the program declares
`L0424` refuses `map.get` at a key the program declares, because the instance is not the library's.
A compiler keys its tables by names, spans, and type variables, and every one is a declared type.
[063][a] - `docs/specs/codegen.md` states how a library generic reaches the program's instance.
[063][b] - `map.insert` at a key the program derives `Eq` for compiles, and `map.get` reads it back.
[063][c] - `L0424` is retired from `docs/specs/diagnostics.md`, or narrowed to what still holds.

## 🔴 Item 064: An instance is written over a generic type
`Eq<List<T>>` cannot be written, and `derive` refuses a record that holds a `List`.
An AST, a `Type`, and a constant pool each hold lists and each needs equality and hashing.
[064][a] - `docs/specs/traits.md` states an instance for a type written with type parameters.
[064][b] - The library writes `Eq`, `Ord`, `Hash`, and `Show` for `List<T>`, constrained on `T`.
[064][c] - `derive` reaches through a field of type `List<T>` when `T` has the instance.
[064][d] - `L0422` fires only where the element type has no instance.

## 🔴 Item 065: A hashed map and set
**Depends on:** Item 063, Item 064 — a table bucketed by `Hash<K>` needs the program's own keys.
`Map` is a linked list, and a compiler with thousands of names is quadratic over it.
`docs/specs/collections.md` says the table waits on an array, which `extern` may name by then.
The table is a textbook hash table: constant time on average, plain, correct, and not tuned.
[065][a] - `docs/specs/collections.md` states the table, the `Hash<K>` constraint, and each cost.
[065][b] - `map.get` and `set.has_value` are constant time over a full table.
[065][c] - Every executable example under `tests/spec/library` still passes unchanged.

## 🔴 Item 076: A list grows in amortized constant time
**Depends on:** Item 058 — the two functions land there, and this item changes what carries one.
`docs/specs/library.md` writes down that `push` costs what the list holds.
A list is one `java.util.List`, and a push copies the whole of it.
A textbook list grows in amortized constant time, which asks for a buffer and a length beside it.
[076][a] - `docs/specs/codegen.md` states what carries a list, and what a push does to it.
[076][b] - `push` costs amortized constant time, and `at` still costs the same at every index.
[076][c] - A push leaves the list it was handed holding what it held, which stays a property.
[076][d] - `docs/specs/library.md` drops the paragraph that writes the copy cost down.

## 🔴 Item 066: The lexer is written in Bux
**Depends on:** Item 059, Item 076 — a lexer builds a token list from the code units of a string.
Self-hosting starts with the smallest phase, and the lexer is 286 lines of Rust.
The Rust `bux` compiles the Bux lexer, and `tests/spec/lexer` holds both to one answer.
[066][a] - `compiler/lexer.bx` lexes a module into the tokens `docs/specs/lexer.md` states.
[066][b] - A harness runs the Bux lexer over every `tests/spec/lexer` fixture and compares tokens.
[066][c] - `docs/implementation.md` section 6 names `compiler/` as where the Bux compiler lives.

## 🔴 Item 067: The parser and the AST are written in Bux
**Depends on:** Item 066 — the parser consumes the tokens the Bux lexer gives.
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

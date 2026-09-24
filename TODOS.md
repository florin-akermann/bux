# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 116: The lowering lowers the modules of one pass at the same time
**Depends on:** Item 111, Item 115 — the profile prices lowering, and 115 pools the phases.
Attacks: lowering, 0.46 s of the 3.5 s of `bux build compiler/main.bx`, 13% (section 7).
Item 123: one pass lowers a build now; a module lowered alone sends its asks to a second pass.
`pass_over` in `compiler/ir.bx` lowers each module in turn.
The asks of one module reach the next module of the same pass.
A pass that gives every module the asks known when the pass starts lowers each module alone.
The passes repeat until no module asks for more, as they do now, and the result is the same.
`bin/bootstrap` holds the classes byte for byte, so it is the check that the order changed nothing.
Item 091 measured the writer at 0.31 s, so it stays on one thread unless the profile says otherwise.
[116][a] - `docs/implementation.md` section 6 states that a pass lowers each module alone.
[116][b] - `pass_over` hands each module of a pass to the pool.
[116][c] - A drawn property holds that a program lowered in a pool gives the classes of one thread.
[116][d] - Section 7 records `bux build compiler/main.bx` and `bin/bootstrap` before and after.

## 🔴 Item 127: `String` is a Bux value over bytes, and `Eq<String>` runs Bux code
`docs/implementation.md` section 12 states the change and asks for the cost first.
`java.lang.String` carries a `String`, and it is the one Java class under a Bux value.
The prelude writes `Eq<String>` as Bux code, but the lowering puts `String.equals` under it.
So one prelude type has a power a declared type lacks, against `docs/design.md` section 3.
A `String` over its bytes moves that logic into `library/strings.bx`, written once in Bux.
`compiler/bytes.bx` already holds a run of bytes as a `List<Int>`, so the bytes need no new type.
[127][a] - Section 7 prices `==`, `length`, `cut_out`, and `+` over `java.lang.String` and bytes.
The price is the build of the compiler before and after.
[127][b] - The item stops at [a], and section 12 records the numbers, when the build is slower.
[127][c] - `library/strings.bx` declares `String` as a record over bytes, and `Eq<String>` as Bux.
[127][d] - The lowering puts nothing under `Eq<String>`, and an `extern` with text converts it.
[127][e] - `bin/bootstrap` gets a new seed, as section 6 reason two states.
[127][f] - `tests/spec/library/` holds, and a drawn property round trips bytes through `String`.

## 🔴 Item 131: The profile of `1brc/src/main.bx` says what the next item attacks
**Depends on:** Item 130 — the program must run over one billion rows before it is profiled.
Section 7 profiled the compiler, and each item after Item 111 attacked a share the profile priced.
The program gets the same, and no library change lands before the profile prices it.
`docs/specs/collections.md` refuses a tuned node until a measurement on a real program asks.
`map.insert` copies one node of thirty-two children at each level.
One row is one `get` and one `insert`, so one row copies about a hundred children.
`strings.at` guards two bounds and one `extern` for each byte.
`strings.cut` makes one `String` for each line, and `Hash<String>` hashes it once for each row.
Those are the candidates the reading suggests, and the profile says which one is a cost.
[131][a] - Section 7 records the profile with JFR over one billion rows, by module and by class.
It records the wall time, the processor time, and the time the collector paused.
[131][b] - The item files one item for the largest share, with the requirement and the alternatives.
It files no item for a share below ten percent, and it changes no library code.

## 🔴 Item 133: The compiler's sources move to `src/`, and its resources sit beside `library/`
**Depends on:** Item 113, Item 114 — both rewrite the class path in `bin/`, which this item moves.
Item 134 holds every package to one layout: `src/` modules, `tests/` tests, and `target/` classes.
The compiler is a package, so that rule refuses the compiler until the compiler keeps the layout.
So the move lands first, whole, with no new rule, and the rule lands on a tree that keeps it.
`compiler/` holds the modules of the compiler, its manifest, `help/`, and `explanations/`.
The help text, the explanations, and `library/` are resources of the toolchain, in no package.
They sit at the root, beside each other, and a build copies the three as Item 113 states.
A moved source changes no class, so the seed stays, and `bin/bootstrap` shows that nothing changed.
[133][a] - `compiler/*.bx` and `compiler/bux.package` move to `src/`; the compiler is `src/main.bx`.
[133][b] - `compiler/help/` and `compiler/explanations/` move to `help/` and `explanations/`.
[133][c] - `tests/bux.package` depends on `../src`.
[133][d] - `bin/bux` starts the classes of `src/target/`, and `bin/bootstrap` builds `src/main.bx`.
[133][e] - Each doc comment, spec, golden, and `docs/implementation.md` names the new path.
[133][f] - `bin/bootstrap` holds stage 2 equal to stage 1 byte for byte across the move.

## 🔴 Item 134: A package is `src/`, `tests/`, and `target/` under its manifest
**Depends on:** Item 133 — the compiler is a package, and it moves before the rule can refuse it.
A package is one flat directory today, a test has no place of its own, and `target/` sits beside it.
So every project is laid out its own way, and a reader learns each one.
One layout, which the compiler holds every package to, is one thing to learn and nothing to choose.
A Go or a Cargo reader expects it: `src/`, `tests/`, and `target/` under the manifest.
`docs/` is a convention, and the compiler reads nothing under it, so it refuses nothing about it.
A directory the compiler holds nothing to is a check that protects nothing, and there is none.
`docs/specs/packages.md` states the rule in a section of its own, and every other page points there.
A module of a package is a `.bx` file at the top of `src/`.
A test module is a `.bx` file at the top of `tests/`.
A test module reaches a module of `src/` by its name, and a module of `src/` reaches no test module.
A `depends` reaches the `src/` of the dependency, and never its `tests/`.
A `.bx` file beside the manifest is `L0322`, at the file, and its help names `src/` and `tests/`.
A manifest inside `src/` or `tests/` of a package is `L0322` too, because a `tests/` is no package.
A `.bx` file deeper than the top of `tests/` is data, as the spec files under `tests/spec/` are.
`bux test <dir>` runs the modules of `src/` and then those of `tests/`, each in sorted name order.
`bux build`, `bux run`, and `bux test` write into `target/` under the manifest of the package.
A bare module, in no package, is unchanged: it writes `target/` beside itself.
The `tests/` of the repository becomes the tests of the root package, and `tests/bux.package` goes.
`example/` becomes a package, so a newcomer reads the layout in the program a newcomer reads.
No module imports `main`, so the types and functions of the example move to `example/src/orders.bx`.
`example/tests/orders.bx` holds one `test` block that reaches `orders`, which shows the import rule.
[134][a] - `docs/design.md` section 16 and `docs/specs/packages.md` state the layout and `L0322`.
`docs/specs/modules.md`, `run.md`, `testing.md`, and `example-program.md` are rewritten to match.
[134][b] - The loader answers an import of a test module from `src/`, and never the other way.
[134][c] - `L0322` refuses a module beside a manifest, and a manifest in `src/` or `tests/`.
[134][d] - A build writes `target/` under the manifest, and `bux test` runs `src/` then `tests/`.
[134][e] - `src/bux.package` moves to the root, and `tests/bux.package` goes.
`bin/bux` starts the classes of `target/`.
`bin/bootstrap` copies the root package, and it compares `target/` with the `target/` of the copy.
The pre-commit hook runs `bin/bux test` at the root.
[134][f] - `example/` holds `bux.package`, `src/main.bx`, `src/orders.bx`, and `tests/orders.bx`.
The README and `tests/started.bx` run `example/src/main.bx`.
[134][g] - `tests/spec/packages/` shows each refusal, and a drawn package in the layout loads.
`tests/commands/fixtures.txt` holds the goldens of each command on a package in the layout.

## 🔴 Item 136: A runner, a build, and a run use a bounded share of the machine
The compiler's own JVMs have no bound, so two runners at once held 8 GB on 2026-09-24.
`bin/runner`, `bin/bux`, and `bin/bootstrap` start a JVM with no `-Xmx`.
JDK ergonomics give each JVM a quarter of the memory, 6 GB, and G1 grows to it before it collects.
So a runner held 4 GB, a `bux build compiler/main.bx` held 1.4 GB, and the bound is per JVM.
Nothing bounds the sum, and each worktree agent starts runners of its own beside the others.
A pass starts 401 short JVMs, one for each run, each with the flags of a long program.
Each gets G1 with 10 collector threads, C2 with 4 compiler threads, and no class-data sharing.
A pool of 12 workers starts up to 12 of them at once, on 12 processors that other pools share.
Section 7 priced a start at 0.064 s, but not what the 12 at once cost the runner's own JIT.
The count of JVMs is by design since Item 084, and this item does not change it.
It bounds what one JVM takes, and it measures whether a smaller pool costs wall time.
[136][a] - `docs/implementation.md` section 7 records the RSS of each JVM and the pass wall time.
It records them before and after, at the load the run had, with the flags of each JVM.
[136][b] - `bin/runner`, `bin/bux`, and `bin/bootstrap` start the compiler with a heap bound.
The measurement of [136][a] says what the bound is.
[136][c] - `command.program_line` starts each run with the flags of a short program.
Class-data sharing, the serial collector, and C1 only are tried, and each stays where it measures.
The runner starts its runs with the same line, so `bux test` and the runner agree.
[136][d] - A measurement says what a pool of half the processors costs in wall time.
The smaller pool stays where the cost is small, and the measurement is recorded either way.

## 🔴 Item 137: A comment above a declaration says why, or it is not there
Nearly every declaration in the Bux sources carries a `///` comment that says what it gives back.
2372 of the 2460 functions of `compiler/`, `library/`, and `1brc/` carry one, and `tests/` alike.
Such a comment restates the signature, and it drifts when the body changes.
The code and its `// example:` lines carry the what, and a comment is kept only for a why.
A why is a constraint the types cannot state, a trade-off, a platform fact, or a decision.
So `every_punctuation` keeps "longest first, so that `==` wins over `=`", and `lex` keeps nothing.
A kept why is one or two `//` lines above the examples, and no `///` line remains.
`docs/specs/doc-examples.md` requires the examples, and `bux test` runs them, so each one stays.
The file header stays, and a comment inside a body stays.
The `.tokens`, `.ast`, `.error`, and `.api` goldens hold byte offsets, so their `.bx` files stay.
`tests/spec/format/` shows the formatter moving a `///` line, so that directory stays.
[137][a] - `AGENTS.md` states the rule: a comment above a declaration says why, or it is not there.
[137][b] - Every `///` line above a declaration in `compiler/`, `library/`, and `1brc/` goes.
A kept why is rewritten as `//` lines, and `bin/bux check` accepts each file.
[137][c] - Every `///` line above a declaration in `tests/*.bx` goes the same way.
[137][d] - Every `///` line above a declaration in `tests/spec/` goes, except where a golden stays.
[137][e] - `bin/bootstrap` and `bin/runner` pass, and `mycs check` reports no finding.

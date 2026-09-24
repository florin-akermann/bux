# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 116: The lowering lowers the modules of one pass at the same time
**Depends on:** Item 111, Item 115 — the profile prices lowering, and 115 pools the phases.
Attacks: lowering, 0.46 s of the 3.5 s of `bux build src/main.bx`, 13% (section 7).
Item 123: one pass lowers a build now; a module lowered alone sends its asks to a second pass.
`pass_over` in `src/ir.bx` lowers each module in turn.
The asks of one module reach the next module of the same pass.
A pass that gives every module the asks known when the pass starts lowers each module alone.
The passes repeat until no module asks for more, as they do now, and the result is the same.
`bin/bootstrap` holds the classes byte for byte, so it is the check that the order changed nothing.
Item 091 measured the writer at 0.31 s, so it stays on one thread unless the profile says otherwise.
[116][a] - `docs/implementation.md` section 6 states that a pass lowers each module alone.
[116][b] - `pass_over` hands each module of a pass to the pool.
[116][c] - A drawn property holds that a program lowered in a pool gives the classes of one thread.
[116][d] - Section 7 records `bux build src/main.bx` and `bin/bootstrap` before and after.

## 🔴 Item 127: `String` is a Bux value over bytes, and `Eq<String>` runs Bux code
`docs/implementation.md` section 12 states the change and asks for the cost first.
`java.lang.String` carries a `String`, and it is the one Java class under a Bux value.
The prelude writes `Eq<String>` as Bux code, but the lowering puts `String.equals` under it.
So one prelude type has a power a declared type lacks, against `docs/design.md` section 3.
A `String` over its bytes moves that logic into `library/strings.bx`, written once in Bux.
`src/bytes.bx` already holds a run of bytes as a `List<Int>`, so the bytes need no new type.
[127][a] - Section 7 prices `==`, `length`, `cut_out`, and `+` over `java.lang.String` and bytes.
The price is the build of the compiler before and after.
[127][b] - The item stops at [a], and section 12 records the numbers, when the build is slower.
[127][c] - `library/strings.bx` declares `String` as a record over bytes, and `Eq<String>` as Bux.
[127][d] - The lowering puts nothing under `Eq<String>`, and an `extern` with text converts it.
[127][e] - `bin/bootstrap` gets a new seed, as section 6 reason two states.
[127][f] - `tests/spec/library/` holds, and a drawn property round trips bytes through `String`.

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

## 🔴 Item 137: A comment above a declaration says why, or it is not there
Nearly every declaration in the Bux sources carries a `///` comment that says what it gives back.
2372 of the 2460 functions of `src/`, `library/`, and `1brc/` carry one, and `tests/` alike.
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
[137][b] - Every `///` line above a declaration in `src/`, `library/`, and `1brc/` goes.
A kept why is rewritten as `//` lines, and `bin/bux check` accepts each file.
[137][c] - Every `///` line above a declaration in `tests/*.bx` goes the same way.
[137][d] - Every `///` line above a declaration in `tests/spec/` goes, except where a golden stays.
[137][e] - `bin/bootstrap` and `bin/bux test tests` pass, and `mycs check` reports no finding.

## 🔴 Item 138: `map.with_child` copies a node without a fresh row of empty children
**Depends on:** Item 131 — its profile priced `map.insert` at 82% of the billion-row run.
Section 7 profiled `1brc/src/main.bx`, and `map.insert` is 82% of its samples in two runs.
Of it, `map.with_child` is 69%: each level of the walk copies a node of 32 children.
Its `for` loop runs over `no_children()`, a fresh row of 32 empty maps, only to count to 32.
That row is 26% of the run, and `Map.Empty` is 25% of the allocation pressure.
The requirement is the processor time of the run, 874 s to 915 s, which the node copy prices.
The alternatives weighed:
- `with_child` loops over the `children` it copies, which removes the row and adds nothing.
- The compressed node of a bitmap, which `docs/specs/collections.md` weighed and refused.
  It narrows a node to the children it holds, but it adds a bit operation and a population count.
- A `list` function that puts one element in place of another; a `for` loop writes it.
- One walk for `get` and `insert` together, which adds a second spelling of an update.
The first is the smallest and asks for no new concept; the profile after it says if more is needed.
[138][a] - `map.with_child` copies a node with no call to `no_children`.
[138][b] - Section 7 records the processor time of the run before and after, at a load below 12.
[138][c] - The examples of `library/map.bx` and `tests/spec/library/` hold.

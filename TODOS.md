# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 114: The runner's checks become `test` blocks, and `bin/runner` goes
**Depends on:** Item 112, Item 113 — the pool and the resources must be in `bux test` first.
`tests/runner.bx` calls six parts, and each part is a function of a module under `tests/`.
Each part becomes one or more `test` blocks of its module, and `bin/bux test tests` runs them all.
Then there is one test runner in the project, and it is the one every Bux program has.
A part that started JVMs still does, from inside its test, and the limit of 60 s stays.
[114][a] - `docs/specs/testing.md` states that the compiler's tests are the tests of `tests/`.
`docs/implementation.md` section 7 is rewritten to match.
[114][b] - Each module of `tests/` that the runner calls states its checks as `test` blocks.
[114][c] - `bin/runner golden` becomes `bin/bux run tests/golden.bx`, which rewrites the goldens.
[114][d] - `.githooks/pre-commit` runs `bin/bootstrap` and then `bin/bux test tests`.
[114][e] - `bin/runner` and `tests/runner.bx` are deleted.
[114][f] - Where one test of every spec example is slower than the chunks were, one module per area.
[114][g] - Section 7 records the wall time of the suite before and after.

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

## 🔴 Item 120: A `private` declaration exists, and only a public function carries an example
Every name a module declares is public, so every helper is API, and every helper pays an example.
`compiler/exhaustiveness.bx` shows the cost.
Five helpers exist only to keep the examples of other functions on one line.
They are `said_in`, `found_printed`, `no_reading`, `no_space`, and `status_module`.
`docs/rationale.md` section 11 says the `test` block was added to prevent that shape.
One keyword is a smaller surface than the helpers it removes, and `bux api` lists less.
[120][a] - `docs/design.md` sections 11 and 16 state `private` and narrow the example rule.
[120][b] - `docs/specs/modules.md` states that a `private` name is reached only in its module.
`docs/specs/api-surface.md` leaves a private declaration out of the page.
[120][c] - `docs/specs/doc-examples.md` requires an example of a public function only.
A private function may state one, and `bux test` runs it.
[120][d] - The lexer, parser, formatter, and resolver carry `private`.
An import that reaches a private name is refused with a code and a help that names the module.
[120][e] - `tests/spec/modules/` and `tests/spec/examples/` show the refusal and the exemption.
[120][f] - The helpers of `compiler/exhaustiveness.bx` named above become `private` or tests.


## 🔴 Item 126: The lowering names no JVM shape, and `compiler/jvm.bx` spells every one
`compiler/ir.bx` names a class, a descriptor, a slot, or a `java/` member on about 160 lines.
It builds the instructions of the JVM, and `compiler/jvm.bx` only writes them as a class file.
So what a construct means and how the JVM spells it are one text, and each reads harder for it.
`spawned` in `compiler/ir.bx` writes `startVirtualThread` beside numbered slots and descriptors.
Item 125 fixes a bug in `offer_within` and `ended_test`, and a named local shows such a bug.
A lowering that names a local and a call reads as what a `process` means.
A JVM phase that gives each local a slot and each call a descriptor reads as how the JVM spells it.
A tree whose types have no field for a JVM name makes a `java/` string in the lowering unwriteable.
[126][a] - `docs/implementation.md` section 6 states the two phases, and what each one may name.
[126][b] - `compiler/ir.bx` lowers to a tree whose types have no field for a JVM name or a slot.
[126][c] - `compiler/jvm.bx` gives a local its slot and a call its descriptor, and writes the class.
[126][d] - `bin/bootstrap` holds stage 2 equal to stage 1 byte for byte across the change.
[126][e] - Section 7 records `bux build compiler/main.bx` before and after.

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

## 🔴 Item 128: The library declares every `extern`, and the compiler declares none
98 `extern` declarations exist: 59 in `library/` and 39 in `compiler/`.
`compiler/command.bx` declares 17, `compiler/archives.bx` 10, and `compiler/modules.bx` 6.
Three more files declare the six others.
`java.io.File` is declared three times.
It is `File` in `library/files.bx` and `compiler/modules.bx`, and `Entry` in `compiler/command.bx`.
`docs/design.md` section 2 gives one thing one spelling, and three for one class break it.
`docs/implementation.md` section 3 says a program reaches the platform through the library.
The compiler is a program, and it is the first one the rule holds to.
[128][a] - Section 4 lists what the library stands on: files, processes, the clock, and streams.
[128][b] - Each `extern` of `compiler/` moves to its library module, or one there replaces it.
[128][c] - A project check refuses an `extern` outside `library/`, and its message names section 3.
[128][d] - Section 4 records the count of `extern` declarations before and after.

## 🔴 Item 130: `1brc/src/main.bx` answers the One Billion Row Challenge
**Depends on:** Item 129, Item 134 — a worker reads part of the file, and 1brc keeps the layout.
The challenge is a file of one billion `<station>;<temperature>` lines, and one line of output.
The output is `{Abha=-23.0/18.0/59.2, Abidjan=-16.2/26.3/67.3, ...}`, sorted by station name.
Each station shows its lowest, mean, and highest reading, each with one decimal.
A half rounds toward positive infinity, as `Math.round` rounds, which the challenge states.
A name is UTF-8 of at most 100 bytes, and a reading is `-99.9` to `99.9` with one decimal.
There are at most 10 000 stations.
`example/src/main.bx` is the program a newcomer reads, and `1brc/src/main.bx` is the one measured.
It is the second dogfood program, written in Bux over `library/` alone, with no `extern` of its own.
`main` reads the path from its arguments, and spawns one worker process for each processor.
Each worker takes one contiguous part of the file, and reads it in slices with `read_between`.
It starts after the first line end past its start, and ends after the first past its end.
A reading is held in tenths as an `Int`, so no floating type is needed.
The mean, and its rounding, are whole-number arithmetic.
A worker holds a `map.Map<String, Summary>` and the list of the names it has seen.
`main` takes each summary with `ended`, merges them by the names, sorts the names, and writes.
Dogfooding found four gaps, and question 12 of `docs/principles.md` judges each one.
A whole number read off text is a `for` loop over `strings.at`, so the program writes it.
An iterator over a map is the list of names the program holds, so nothing lands.
A sort is the loop `sorted` in `compiler/command.bx` writes, and the program would write it twice.
`docs/specs/library.md` lands a function on that test, so `list.sorted` lands, at the cost asked.
The count of processors is what `tests/runner.bx` reaches with an `extern` of its own.
So that count moves to the library.
[130][a] - `docs/specs/billion-rows.md` states the program: input, output, exit codes, and `1brc/`.
[130][b] - `list.sorted<T: Ord<T>>(values: List<T>) -> List<T>` lands, a merge sort with loops.
`sorted` and `inserted` in `compiler/command.bx` go, and `command.bx` calls `list.sorted`.
[130][c] - `environment.processors() -> Int` lands, and `tests/runner.bx` calls it.
[130][d] - `1brc/src/main.bx` is the program, and `1brc/tests/samples/` holds samples and outputs.
One sample holds UTF-8 names, one holds a mean that is a negative half, and one holds one station.
[130][e] - `tests/started.bx` runs `1brc/src/main.bx` over each sample, as it runs the example.
It holds the output to the expected one, and it runs the examples and tests of the module.
[130][f] - `docs/implementation.md` section 12 names the program as the second dogfood.
Section 7 records the wall time over one billion rows, on the machine and the JDK it names.

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

## 🔴 Item 132: `1brc/src/create_measurements.bx` writes the file of one billion rows
**Depends on:** Item 130 — the program is what reads what this writes.
The challenge gives a Java generator, and a JDK runs it, so the file exists before this item.
Dogfooding says Bux writes it, and the generator finds one more gap.
`files.write` writes a file whole, and 13 GB is no `String`, so a file is written in parts.
No `for` loop appends to a file, so `files.append` passes question 12 of the principles.
A draw is a linear congruential generator, as `tests/drawn.bx` writes one.
The program writes its own, because no program imports a module of `tests/`.
The generator holds the stations of the challenge and the mean of each in one list literal.
[132][a] - `docs/specs/io.md` states `files.append(path, text) -> Result<String, String>`.
It writes `text` after what the file holds, makes the file where there is none, and gives `path`.
[132][b] - `docs/specs/billion-rows.md` states the command line of the generator.
It is `bux run 1brc/src/create_measurements.bx <rows> <path>`.
[132][c] - `1brc/src/create_measurements.bx` writes `rows` lines, in parts of one million lines.
[132][d] - A test of the module writes a small file, and `1brc/src/main.bx` reads it to its output.
[132][e] - Section 7 records the wall time of one billion rows written, beside their read time.

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

## 🔴 Item 135: A module that imports itself crashes the compiler
`import list` inside `library/list.bx` ends `bux build` with a `StackOverflowError`.
`modules.walk` follows the import into the module it walks, and it never ends.
The compiler never crashes, so an import of the module itself is a refusal with a code.
[135][a] - `docs/specs/modules.md` states the refusal, its code, and its message.
[135][b] - `modules.walk` refuses a self-import before it follows any import.
[135][c] - `tests/spec/modules/imports_itself.bx` is the example, and the runner holds it.

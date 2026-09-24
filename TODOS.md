# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

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

## 🔴 Item 112: `bux test` runs the modules of a package on a pool of workers
**Depends on:** Item 111 — the profile says what a module run spends on typing before it starts.
Attacks: typing, 2.4 s of the 11.2 s that one job over `compiler/` compiles (section 7).
Item 123: `bux test tests` waited 9.9 s of 177 s on its 43 JVMs, so 94% is compile work to pool.
`every_module_tested` in `compiler/command.bx` runs the modules of a package one after another.
`tests/runner.bx` owns a pool of one worker per processor, and only the compiler's tests use it.
The pool moves into `bux test`, where every package gets it, and the runner keeps working meanwhile.
A worker is a process, and its state is where a memo lives across the modules it runs.
A module of `tests/` imports most of the compiler, so a memo saves about 40 000 lines of typing.
[112][a] - `docs/specs/testing.md` states that the modules of a package run at the same time.
It states one worker for each processor, and a report in file order.
[112][b] - A `process` in `compiler/command.bx` hands each module to a worker.
It holds each answer by the number of its module, as the pool of `tests/runner.bx` does.
[112][c] - `Working` holds the memo, and a worker keeps it from one module to the next.
[112][d] - A drawn property holds that a report on a pool is the report of modules run in turn.
[112][e] - Section 7 records `bux test compiler` and `bux test tests` before and after.

## 🔴 Item 113: The compiler finds its library, help, and explanations beside its classes
`modules.resource_text` reads a file under each entry of the class path.
So `bin/bux`, `bin/bootstrap`, and `bin/runner` add `compiler/` and the root to the class path.
A run that `bux test` starts has only its own directory on the class path.
So a test of `tests/` that calls the compiler for `bux help` or `bux explain` finds nothing.
That is why the runner has a class path of its own, and why it cannot be `bux test tests`.
A build writes the three resource directories into `target/` beside the classes it writes.
Then every class path that holds the classes holds the resources, and no script adds an entry.
[113][a] - `docs/specs/run.md` section "Where a build writes" states what a build writes too.
[113][b] - `bux build` copies `library/`, `help/`, and `explanations/` into `target/`.
`bux test` does the same for each run.
[113][c] - `bin/bux`, `bin/bootstrap`, and `bin/runner` put one directory on the class path.
[113][d] - `docs/implementation.md` section 6 states the class path, and section 7 the runner's.

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

## 🔴 Item 119: `bux fmt` repairs every order the compiler can compute
`docs/specs/formatting.md` says that order is checked and never rewritten.
Import order, the place of a test, and declaration order each have one answer the compiler knows.
Each refusal costs an agent one round trip: write, build, read the refusal, edit, build again.
That round trip costs more than the rule saves, so `bux fmt` writes the answer it already knows.
Naming stays a refusal, because the compiler cannot choose a name.
Arm order stays a refusal until a case shows that the formatter needs the types.
[119][a] - `docs/design.md` section 13 and `docs/specs/formatting.md` name each order `fmt` repairs.
[119][b] - `compiler/format.bx` sorts the imports and moves a declaration written after a test.
[119][c] - `bux fmt` runs the resolver and moves a declaration below what uses it.
Two declarations that use each other keep the order the author wrote.
[119][d] - `L0201` and `L0303` stay for `bux build`, and their help names `bux fmt`.
[119][e] - `tests/spec/format/` shows each repair.
A property holds that a repaired file builds, and that `fmt` of a repaired file changes nothing.

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


## 🔴 Item 125: `send` says `Delivered` for a message that the process reads
`tests/spec/concurrency/mailbox_full.bx` sends `Stop` to an `Idle` process and expects `delivered`.
One runner pass of Item 112 printed `the process has ended` for that line, and five printed `delivered`.
`send` offers the message to the mailbox, and then it tests whether the process has ended.
`offer_within` and `ended_test` in `compiler/ir.bx` write the two steps.
`Idle` takes `Stop` and ends between the two steps, so a message the process read is reported as never read.
`docs/specs/concurrency.md` says that a message the mailbox takes after the end gives `ProcessEnded`.
The answer must follow what happens to the message, not the time of the test.
[125][a] - `docs/specs/concurrency.md` states that `send` gives `ProcessEnded` only for a message that no `receive` reads.
[125][b] - A process that ends leaves its mailbox in a state `send` can tell from a taken message.
[125][c] - `tests/spec/concurrency/mailbox_full.bx` holds on every pass of the runner.

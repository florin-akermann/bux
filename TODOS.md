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

## 🔴 Item 110: `bin/bootstrap` compares the two stages with one `diff`, not one `cmp` per class
On 2026-09-24 `bin/bootstrap` took 7.3 s, and its loop of 918 `cmp` calls took 2.6 s of them.
One `diff -rq` over the two `target/` directories takes 0.03 s and names every file that differs.
The script keeps its contract: it names the first class that differs and ends with status 1.
[110][a] - `docs/specs/run.md` section "The bootstrap" states that one `diff` compares the stages.
[110][b] - `bin/bootstrap` runs one `diff -rq`, and it names the first class of the answer.
It still refuses with status 1 when one stage holds a class the other does not.
[110][c] - `docs/implementation.md` section 7 records the wall time before and after.

## 🔴 Item 111: A profile says where a build and a runner pass spend their time
Item 091 put the class writer on 1400 processes and measured no gain, because the guess was wrong.
So the speed-up starts with a measurement, and every item after this one cites it.
On 2026-09-24 a JVM that prints `bux help` took 0.12 s.
`bux check compiler/main.bx` took 2.0 s to 2.8 s.
`bux build compiler/main.bx` took 2.8 s to 3.3 s.
The same build with `-XX:TieredStopAtLevel=1` took 4.2 s, so the time is work and not JIT waiting.
A class data archive of the compiler's classes changed nothing, so class loading is not the cost.
`bin/runner` took 62 s of wall time and 311 s of processor time, on 12 workers.
Its jobs of example lines took 326 s of that, its command lines 44 s, its examples 18 s.
[111][a] - JFR, which ships with the JDK, records one `bux build compiler/main.bx`.
`docs/implementation.md` section 7 records the seconds of each phase, from the lexer to the writer.
[111][b] - JFR records one job of example lines over the modules of `compiler/`.
Section 7 records the seconds of typing, of lowering, of class writing, and of the JVM starts.
[111][c] - Section 7 records how many JVMs one runner pass starts, and what one start costs.
[111][d] - Each of Items 112 and 115 to 117 names the number of this profile it attacks.
An item whose number is small is removed from `TODOS.md` rather than built.

## 🔴 Item 112: `bux test` runs the modules of a package on a pool of workers
**Depends on:** Item 111 — the profile says what a module run spends on typing before it starts.
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

## 🔴 Item 115: The compiler types independent modules at the same time
**Depends on:** Item 111, Item 112 — the profile prices inference, and 112 holds the pool.
`each_accepted` in `compiler/command.bx` types the modules one after another in load order.
A module needs only the surface of each module it imports, so modules of one wave are independent.
The pool of Item 112 types every module whose imports are typed.
The refusal a build reports stays the first one in load order, whatever process ends first.
[115][a] - `docs/specs/types.md` states that the reported refusal does not depend on the order.
[115][b] - The pool hands a module to a worker once its imports are typed.
[115][c] - A drawn property holds that a program typed in waves is typed as it is in load order.
[115][d] - Section 7 records `bux check` and `bux build` on `compiler/main.bx` before and after.
It records `bin/bootstrap` with them.

## 🔴 Item 116: The lowering lowers the modules of one pass at the same time
**Depends on:** Item 111, Item 115 — the profile prices lowering, and 115 pools the phases.
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

## 🔴 Item 117: A run of `bux test` writes only the classes its run adds
**Depends on:** Item 111, Item 114 — the profile says what a run spends on its classes.
Each run writes the classes of its whole import closure into a directory of its own.
A run of a module of `tests/` writes about 900 classes, and 73 modules do so in one suite.
`docs/implementation.md` section 7 says why one shared build is not correct.
A run asks generics for types that no build asks for, and each lands in the class that declares it.
The classes a run changes are those of the modules that declare an asked generic, and no other.
[117][a] - `docs/specs/testing.md` states which classes a run shares with the run before it.
[117][b] - A worker writes a class again only where its bytes differ from the last run it made.
Each run still starts a JVM with a class path of its own.
[117][c] - A property holds that the class path of a run holds every class its program reaches.
[117][d] - Section 7 records the wall time of `bux test tests` before and after.

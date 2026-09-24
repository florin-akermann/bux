# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

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
[111][d] - Each of Items 112 to 115 names the number of this profile it attacks.
An item whose number is small is removed from `TODOS.md` rather than built.

## 🔴 Item 112: The runner types the compiler once for each worker, not once for each module
**Depends on:** Item 111 — the profile says what a job spends on typing before its first run.
A job of example lines starts with `command.no_memo()`.
A module of `tests/` imports most of the compiler, so a job types about 40 000 lines first.
73 modules have example lines, and the runner splits them into jobs by place and by worker.
A worker is a process, and its state is where a memo lives across jobs.
[112][a] - `docs/implementation.md` section 7 states that a worker keeps its memo across its jobs.
A command still starts with an empty memo, so no answer of the command line depends on one.
[112][b] - `Working` in `tests/runner.bx` holds the memo, and `Give` returns it to the worker.
[112][c] - A property holds that the answer of a job is the same with an empty memo and a full one.
[112][d] - Where one typing of the compiler per worker is still a large cost, the pool types once.
It types the closure of `compiler/main.bx` before the first job, and each `Give` carries the memo.
[112][e] - Section 7 records the wall time of `bin/runner` before and after.

## 🔴 Item 113: The compiler types independent modules at the same time
**Depends on:** Item 111 — the profile says what inference costs among the phases.
`each_accepted` in `compiler/command.bx` types the modules one after another in load order.
A module needs only the surface of each module it imports, so modules of one wave are independent.
A pool of processes types every module whose imports are typed, as `tests/runner.bx` pools jobs.
The refusal a build reports stays the first one in load order, whatever process ends first.
[113][a] - `docs/specs/types.md` states that the reported refusal does not depend on the order.
`docs/specs/concurrency.md` names the compiler as the second user of a pool.
[113][b] - A `process` in `compiler/command.bx` hands a module to a worker once its imports are in.
It holds each answer by the number of its module, as the pool of the runner does.
[113][c] - A drawn property holds that a program typed in waves is typed as it is in load order.
[113][d] - Section 7 records `bux check` and `bux build` on `compiler/main.bx` before and after.
It records `bin/bootstrap` with them.

## 🔴 Item 114: The lowering lowers the modules of one pass at the same time
**Depends on:** Item 111, Item 113 — the profile says what lowering costs, and 113 holds the pool.
`pass_over` in `compiler/ir.bx` lowers each module in turn.
The asks of one module reach the next module of the same pass.
A pass that gives every module the asks known when the pass starts lowers each module alone.
The passes repeat until no module asks for more, as they do now, and the result is the same.
`bin/bootstrap` holds the classes byte for byte, so it is the check that the order changed nothing.
Item 091 measured the writer at 0.31 s, so the writer stays on one thread unless the profile says so.
[114][a] - `docs/implementation.md` section 6 states that a pass lowers each module alone.
[114][b] - `pass_over` hands each module of a pass to the pool of Item 113.
[114][c] - A drawn property holds that a program lowered in a pool gives the classes of one thread.
[114][d] - Section 7 records `bux build compiler/main.bx` and `bin/bootstrap` before and after.

## 🔴 Item 115: A run of example lines writes only the classes its run adds
**Depends on:** Item 111 — the profile says what a run spends on the classes of its closure.
Each run writes the classes of its whole import closure into a directory of its own.
A run of a module of `tests/` writes about 900 classes, and 73 modules do so in one runner pass.
`docs/implementation.md` section 7 says why one shared build is not correct.
A run asks generics for types that no build asks for, and each lands in the class that declares it.
The classes a run changes are those of the modules that declare an asked generic, and no other.
[115][a] - Section 7 states which classes a run shares with the run before it in the same worker.
[115][b] - `tests/documented.bx` writes a class again only where its bytes differ from the last run.
Each run still starts a JVM with a class path of its own.
[115][c] - A property holds that the class path of a run holds every class its program reaches.
[115][d] - Section 7 records the wall time of `bin/runner` before and after.

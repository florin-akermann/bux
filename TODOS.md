# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 100: `spawn` starts a process that another module declares
Item 091 found it: `L0800` refuses a `spawn` of a process outside the module that declares it.
A library process, such as a ticker or a bus, is then out of reach, and section 15 wants both.
[100][a] - `docs/specs/concurrency.md` states how a process is reached through a module name.
[100][b] - The resolver reaches a process of another module, and `L0800` keeps its other refusals.
[100][c] - A `tests/spec/concurrency/` example spawns a process that another module declares.

## 🔴 Item 101: A foreign reference given to a process is refused
Item 091 found it: `docs/design.md` section 14 refuses a foreign reference given to a process.
No check holds that rule today, so a `spawn` argument or a message can carry one.
[101][a] - `docs/specs/concurrency.md` states the refusal and its code.
[101][b] - The type phase refuses a `spawn` argument and a message that hold a foreign reference.
[101][c] - A `tests/spec/concurrency/` example shows the refusal.

## 🔴 Item 102: A process that a platform error stops has a stated end, and `Delivered` is exact
Item 091 found it: a JVM error, such as a stack overflow, stops a process and is printed.
`ended` then stops its caller with the same error, so one stopped worker stops `bin/runner`.
Section 15 says no process fails, so the spec must say what such an end is and who learns of it.
A message can also be `Delivered` after `Done`, because the check and the offer are two steps.
[102][a] - `docs/specs/concurrency.md` states the end, and how `ended` and `send` report it.
[102][b] - A `tests/spec/concurrency/` example shows a process that a platform error stops.
[102][c] - `Delivered` means the mailbox took the message before the end, and a property shows it.

## 🔴 Item 103: `bin/runner` prints a part as it ends, and example runs share written classes
Item 091 found it: the pool prints every part line after `ended`, so the run shows nothing for 30 s.
The example-line jobs of `compiler/` and `tests/` write the same classes again, and each takes 27 s.
[103][a] - The runner prints each part line when that part ends, and the line order is stable.
[103][b] - The example-line runs of one module share one set of written classes.
[103][c] - `tests/launched.bx` holds its 2 s window under a full pool, or states a wider one.

# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 103: `bin/runner` prints a part as it ends, and example runs share written classes
Item 091 found it: the pool prints every part line after `ended`, so the run shows nothing for 30 s.
The example-line jobs of `compiler/` and `tests/` write the same classes again, and each takes 27 s.
[103][a] - The runner prints each part line when that part ends, and the line order is stable.
[103][b] - The example-line runs of one module share one set of written classes.
[103][c] - `tests/launched.bx` holds its 2 s window under a full pool, or states a wider one.

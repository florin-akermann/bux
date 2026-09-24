# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 107: A foreign reference cannot escape the function that reaches it
`docs/design.md` section 14 states the escape check, and Item 101 found that it is not built.
A function can still return a value of an `extern` type, and a type can still hold one in a field.
`L0811` guards a process only, so the rule holds for a process and not for the rest of a program.
[107][a] - `docs/specs/interop.md` states where a foreign reference can and cannot go.
[107][b] - A diagnostic refuses a return, a field, or a constructor argument of an `extern` type.
[107][c] - A `tests/spec/interop/` example shows each refused place, and a drawn property holds it.

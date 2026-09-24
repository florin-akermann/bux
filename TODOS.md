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
[137][e] - `bin/bootstrap` and `bin/bux test` pass, and `mycs check` reports no finding.


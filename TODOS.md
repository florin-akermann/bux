# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 057: The example program writes its answer out
`example/main.lm` binds three names it does not use, and a run shows only that it reaches the end.
The file says why: version 0.1 has no way to write a number out, so there is nowhere for an answer
to go.
That reason is gone.
`io.println` writes a line, which `docs/specs/io.md` states, and the `extern` work behind it landed.
The prelude declares `Show<Int>`, so `shown` gives a number the text a JVM writes it as.
A reader who opens `example/main.lm` must therefore see what everyday Lumen shows somebody, and
today the file argues it cannot.
[057][a] - `example/main.lm` imports `io` and writes each answer it works out with `io.println`.
[057][b] - `docs/specs/example-program.md` says what the run writes, and drops the reason it wrote
  nothing.
[057][c] - `README.md` says the same, and the test asserts the standard output as well as the
  exit code.

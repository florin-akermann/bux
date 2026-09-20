Print the long form of one diagnostic code.

Every error the compiler reports carries a code, such as `L0105`, on its first line. The code
is the stable handle on that kind of error: it is assigned once and never reused, so a spec, a
test, or a note to a colleague can cite the code and stay right however the wording changes.

`lumen explain <code>` prints what the code means, why the language refuses it, and what to
write instead, at more length than the one `help:` line of a diagnostic has room for.

The codes are grouped by the phase that raises them: the grammar writes L01xx, canonical form
L02xx, loading and name resolution L03xx, type inference L04xx, and exhaustiveness L05xx.

Exit codes: 0 when the code is one the compiler can raise, and 2 when it is not.

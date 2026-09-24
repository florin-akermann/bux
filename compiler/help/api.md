Print what a module offers, and nothing else.

`bux api` runs the front end over a file and prints one page: every name the file declares at
the top level, with the type it has. That is the whole surface, because every top-level
declaration is public and there is no private one. Learning a signature is then reading a page
rather than reading a file.

A function is printed as its signature and nothing after it: one line, and no body under it.
A type declaration is printed exactly as canonical form writes it in a file, variants and fields
included, so a caller sees what to build and what it may read. An `extern` declaration is printed
whole, the Java name it states included, because it writes no body to leave off. An import is not
on the page: it brings a name in rather than putting one out. Neither is a comment, nor anything
a body binds.

The page lists declarations in the order the file declares them. A file reads top down, and the
page reads the same way, so a reader who knows the file knows where to look.

The page is the named file's alone. A module the file imports is compiled, because the names
reached through it have to have types, but nothing it declares is on the page: `bux api` run
against that module is what prints its surface.

Every function writes its whole signature, so every type printed is a type the file writes.
A file that leaves a type out of a signature is refused with `L0436`, and it has no page.

The same refusals `bux check` gives apply here, with the same diagnostics, because it is the
same front end. A module that holds a hole still has a page: a hole is well typed, and refusing
one is left to `bux build`.

The page goes to standard output and a refusal to standard error, so a page piped somewhere is a
page and never a refusal.

Exit codes: 0 when the page was printed, 1 when the compiler refuses the file, and 2 when the
file cannot be read.

Print what a module offers, and nothing else.

`lumen api` runs the front end over a file and prints one page: every name the file declares at
the top level, with the type it has. That is the whole surface, because every top-level
declaration is public and there is no private one. Learning a signature is then reading a page
rather than reading a file.

A function is printed as its signature and nothing after it: one line, and no body under it.
A type declaration is printed exactly as canonical form writes it in a file, variants and fields
included, so a caller sees what to build and what it may read. An import is not on the page: it
brings a name in rather than putting one out. Neither is a comment, nor anything a body binds.

The page lists declarations in the order the file declares them. A file reads top down, and the
page reads the same way, so a reader who knows the file knows where to look.

The page is the named file's alone. A module the file imports is compiled, because the names
reached through it have to have types, but nothing it declares is on the page: `lumen api` run
against that module is what prints its surface.

Every type printed is the type inference settled on, not the text the author wrote. `fn twice(n)`
is printed `fn twice(n: Int) -> Int` when that is what it is, so a page says the same thing
whether or not a signature was written. A type nothing in the module settled prints as `_`, which
is how a diagnostic spells the same thing; writing the signature is what turns it into a type.

The same refusals `lumen check` gives apply here, with the same diagnostics, because it is the
same front end. A module that holds a hole still has a page: a hole is well typed, and refusing
one is left to `lumen build`.

The page goes to standard output and a refusal to standard error, so a page piped somewhere is a
page and never a refusal.

Exit codes: 0 when the page was printed, 1 when the compiler refuses the file, and 2 when the
file cannot be read.

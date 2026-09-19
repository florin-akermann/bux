# The public surface of a module

## Intent

`lumen api <file>` prints what a module offers, and nothing else.
One page then replaces reading a file to learn a signature.

A reader who wants to call something needs the name, what it takes, and what it gives back.
A file holds that among the bodies, the comments, and the order the author wrote them in.
The page holds only the first three, so learning a signature is reading a page rather than a file.

The page is also what a reader consults to see the types inference gave a module.
A signature is written where it documents a boundary, which leaves the rest inferred, and the page
is where the inferred ones are stated.

## What is public

Every name a file declares at the top level is public.
`docs/specs/modules.md` states that there is no private declaration, so nothing is left out.

The page therefore holds every type the file declares and every function it declares.
It holds nothing else.

An import is not on the page: `import io` brings a name in rather than putting one out.
The module that was imported has a page of its own, and that is where its names are stated.

A comment is not a name, so no comment reaches the page.
A name inside a body is not public either: a binding, a loop variable, and a name a pattern binds
each live and die inside the function that writes them.

A variant and a record field are on the page as part of the declaration that declares them, which
is how a caller learns what to build and what it may read.

## The order

The page lists what the file declares, in the order the file declares it.

A file reads top down: the reader meets the intent before the detail, which `docs/design.md`
section 13 requires.
The page reads the same way, so a reader who knows the file knows where to look in the page.
Sorting would put the detail first as often as not, and would make the page a second order to
learn rather than a view of the one that is already there.

## The layout

One declaration is one stanza, and exactly one blank line separates two stanzas, which is the rule
`docs/specs/formatting.md` gives a file.
The page ends with a newline, and an empty page is empty: no blank line, no heading.

A type declaration is written exactly as canonical form writes it in a file.
A type declaration is all surface — it has no body to leave out — so a module that declares only
types is its own API page, character for character.
One printer writes both, so the page can never drift from the form a file is held to.

A function is written as its signature and nothing after it.
`fn shared(total: Int, people: Int) -> Int` is the canonical `fn` line without the ` {` that opens
the body.
A function that declares type parameters keeps them: `fn first<A, B>(pair: Pair<A, B>) -> A`.

A signature is spelled the way the language spells it, wherever the language has a spelling.
Version 0.1 has no syntax for a function type, so a parameter inference made a function is printed
the way a diagnostic prints one, as `(Int) -> Bool`.
Nothing can call such a function: version 0.1 reaches a function by calling it, so a function is
never an argument, and a function that takes one is a function nothing can reach.
The page says so rather than leaving the reader to work it out.

## The types are the inferred ones

Every type on the page is the type inference settled on, not the text the author wrote.

A parameter the author annotated and one they left to inference therefore read the same.
`fn twice(n) { n + n }` is on the page as `fn twice(n: Int) -> Int`, which is what it is.

A type parameter is printed by the name its declaration gave it, because that is the name the
caller reads in the rest of the signature.

A type that nothing in the module settled is printed `_`, which is how a diagnostic spells the
same thing.
Two `_` on one line are not necessarily the same type: the page states what the module states, and
a module that states nothing there has nothing for the page to print.
Writing the signature is what turns a `_` into a type, and it is the author's to write.
`_` is reserved for the discard of `docs/specs/discarding.md` and is no type either, so the page
is a thing to read rather than a signature to paste.

## Exit codes

```text
0  the page was printed
1  the compiler refused the file
2  the file could not be read
```

The page goes to standard output and a refusal goes to standard error, so a page piped somewhere
is a page and never a refusal.

`lumen api` runs the same front end `lumen check` runs, and refuses what it refuses, with the same
diagnostic.
A module that holds a hole has a page: a hole is well typed, and `docs/specs/holes.md` leaves
refusing one to `lumen build`.

## Properties

These hold and are checked with property-based tests:

1. A module of types alone, holding no comment, is its own page, character for character.
2. Every name a module declares at the top level is on its page.
3. Printing a page never panics and is deterministic.

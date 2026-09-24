# The public surface of a module

## Intent

`bux api <file>` prints what a module offers, and nothing else.
One page then replaces reading a file to learn a signature.

A reader who wants to call something needs the name, what it takes, and what it gives back.
A file holds that among the bodies, the comments, and the order the author wrote them in.
The page holds only the first three, so learning a signature is reading a page rather than a file.

Every function writes its signature, so the page states each type without a body to read.

## What is public

Every name a file declares at the top level is public, unless its declaration is `private`.
`docs/specs/modules.md` states that a private name is reached only in its module.

The page therefore holds every public type and every public function the file declares.
It holds nothing else.

A private declaration is not on the page, because no other module can write its name.
An instance and a derive of a private type or of a private trait are not on the page either.
Each of them names something that no page states.
A public function can take or give back a value of a private type, and its signature says so.
That type is not on the page, and a caller holds such a value without writing its name.

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

A signature is spelled the way the language spells it.

## The types are the written ones

Every function writes its whole signature, which `docs/specs/types.md` states.
`L0436` refuses a module that leaves a type out, so that module has no page.
Every type on the page is therefore the type the signature writes, and no page holds a `_`.
The page prints the type inference settled on, which for a signature is the written one.

A type parameter is printed by the name its declaration gave it, because that is the name the
caller reads in the rest of the signature.

## Exit codes

```text
0  the page was printed
1  the compiler refused the file
2  the file could not be read
```

The page goes to standard output and a refusal goes to standard error, so a page piped somewhere
is a page and never a refusal.

`bux api` runs the same front end `bux check` runs, and refuses what it refuses, with the same
diagnostic.
A module that holds a hole has a page: a hole is well typed, and `docs/specs/holes.md` leaves
refusing one to `bux build`.

## Properties

These hold and are checked by drawn properties, each a test of `tests/`:

1. A module of types alone, holding no comment, is its own page, character for character.
2. Every public name a module declares at the top level is on its page, and no private one is.
3. Printing a page never panics and is deterministic.

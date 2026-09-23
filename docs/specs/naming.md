# Naming

Canonical form covers how a name is spelled, not only where it is written.

## Intent

A reader meets a name before they meet what it does.
`UserID` and `UserId` are the same type spelled two ways, and a codebase that allows both spends
attention on which one this file chose.

mycs reports `Cryptic Public Identifier`, `Acronym Casing`, and `Boolean Predicate Prefix` after
the fact, and the file still compiles.
Canonical form makes them errors, as it makes every other spelling decision an error: there is one
way to write a program, and the compiler writes it.

The rules are about declared names, which is every name another file can write.
A local binding is private to the body it is written in, so nothing here is about one.
A parameter name is written at a call site, which `docs/specs/arguments.md` states, so it is
declared rather than private.

## Case

A function, a parameter, a record field, and an imported module are `snake_case`.
A type, a variant, and a type parameter are `PascalCase`.

`snake_case` is lowercase letters and digits, with `_` between words.
A name neither begins nor ends with `_`, and never writes two in a row.

`PascalCase` is words run together, each beginning with one uppercase letter.
**An acronym is a word**: `UserId` is canonical and `UserID` does not compile.
Two uppercase letters never stand next to each other, which is the same rule said the other way.

A name written in neither is `L0202`, and the help prints the canonical spelling of that name.

## A name is a word, not an initial

A declared name is a word of two characters or more.
`f` is an initial: it tells a reader the shape of the code and nothing about the domain.

The word measured is the one canonical form spells, not the characters the author wrote.
`_a` names the one-letter word `a`, and `__` names no word at all, so both are initials too.
Reporting either as a miscased name would advise a spelling that is itself refused.

A type parameter is exempt, because it names no domain concept.
It stands for whatever type a call supplies, and `T` is how that is written.

A name whose word is shorter than that is `L0203`.

## A `Bool` function reads as a predicate

A function whose result is `Bool` answers a question, and its name asks one.

```text
fn is_active(user: User) -> Bool {
    user.active
}
```

The name begins `is_`, `has_`, `can_`, or `should_`.
Four prefixes cover the questions a program asks, and a name that needs a fifth is a name that is
answering more than one.

`active(user)` reads as a command wherever it is called, and `if is_active(user)` reads as the
question it is.
A function named for a command is `L0413`.

A method a trait declares is held to the same rule, and the refusal lands where the trait wrote
the name rather than on the first instance to write a body for it.

The result read is the one inference settled, so a function that writes no result type is held to
whatever type it turned out to have.
That is the same reading `docs/specs/arguments.md` gives the flag rule, and for the same reason: a
signature the author left unwritten counts exactly as one they wrote out.

## The errors

`L0202` is a name written in neither case:

```text
error[L0202]: `UserID` is a type, so canonical form writes it in `PascalCase`
  --> demo.bx:1:6

  1 | type UserID = UserID(Int)
    |      ^^^^^^

help: canonical form spells this name `UserId`
```

`L0203` is a name that is an initial rather than a word:

```text
error[L0203]: `f` is an initial, which names nothing a reader can look for
  --> demo.bx:1:4

  1 | fn f(count: Int) -> Int {
    |    ^

help: a declared name is a word, so write the one this names
```

`L0413` is a `Bool` function named for a command:

```text
error[L0413]: `active` gives back a `Bool`, so its name asks the question it answers
  --> demo.bx:1:4

  1 | fn active(user: User) -> Bool {
    |    ^^^^^^

help: begin the name with `is_`, `has_`, `can_`, or `should_`
```

## Where each rule is checked

Case and length are about the text, so `compiler/format.bx` checks them with the rest of
canonical form, before a name is resolved or a type is settled.
The first name a file writes out of form is the one reported, in the order the file writes them.

The predicate rule reads a type, so `compiler/infer.bx` checks it as inference
settles each function.
It is reached after the body, because the type it reads is the one inference settled.

## The name check written in Bux

`compiler/format.bx` checks case and length with the rest of canonical form.
It gives `L0202` for a name in the wrong case and `L0203` for a name of one letter.
`tests/conventions.bx` holds the name check to the properties below, on drawn modules.

## Properties

These hold and are checked by drawn properties in the runner:

1. A name canonical form rewrites is rewritten to one it accepts, so no spelling it advises is
   refused in its turn.
2. A `Bool` function compiles exactly when its name begins with one of the four prefixes.

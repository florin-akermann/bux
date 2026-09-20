# The library

The prelude and the standard library are Lumen source the compiler carries, not tables it holds.

## Intent

`crates/resolver/src/prelude.rs` says the prelude becomes Lumen source once a module can be
loaded, and a module has loaded since the loader landed.
`docs/implementation.md` section 10 asks for the `Int` and `String` instances of the operator
traits to move into the library.
This spec says where that source lives, how the compiler reaches it, and what is in it.

Dogfooding is the point.
Every table the compiler holds about the prelude is a second way to declare a type, a trait, and
an instance, and a second way drifts from the first.
`Option` written in Lumen is read, formatted, checked, and refused by the same code every other
module is, so there is one declaration of it and nothing to keep in step.

## Where the source is

The library is `library/*.lm` in this repository, one file per module, written as any module is.
It is ordinary Lumen source: canonical form, one name one definition, examples where a function
states one.

The compiler carries it rather than looking for it.
Each file is read into the binary at build time, so a compiler that runs at all has its library:
there is no install layout, no search path, no environment variable, and no directory a command
has to be run from.
That is also what makes every build and every test work with no network and no fixture.

## How it is found

The prelude is read before the module a command names, and before every module that one reaches.
Nothing imports it: its names are in scope in every module, which `docs/specs/modules.md` states,
so an `import prelude` names no module and is refused like any other name nothing holds.

Another library module is reached by importing it, exactly as `io` is.
`import strings` brings `strings` into scope under its own name, and `strings.join(parts, "-")`
is a call of the function `join` that module declares.
An import of a library module looks beside no file: the compiler holds the source, so a file of
that name beside the importing one is not consulted and does not shadow it.

Naming a module of a program `prelude` or `strings` therefore buys nothing.
The name is taken, as `io` and `files` are taken, and an import of one is the library's.

## What the compiler still holds

A type the JVM holds directly stays the compiler's: `Bool`, `Int`, `String`, and `List`.
Nothing a program writes could declare them, because what they are made of is the JVM rather than
any Lumen declaration.
They are named types like any other from where a program stands, which is what
`docs/design.md` section 2 asks: what `Int` can do, a declared type can do.

Everything else the prelude used to supply is library source:

```text
types:        Option  Result
constructors: Some  None  Ok  Err
functions:    or
traits:       Add  Div  Eq  Hash  IntegerLiteral  Mul  Neg  Ord  Rem  Show  Sub
instances:    each of those traits for the types of it the library writes
```

`todo` stays the compiler's, because a hole has no body for the library to write:
`docs/specs/holes.md` has `lumen build` refuse every one of them before a class file is written.

## An operator over a type the compiler holds

`instance Add<Int>` is written in the library, and its body is written in Lumen:

```text
instance Add<Int> {
    fn add(left: Int, right: Int) -> Int {
        left + right
    }
}
```

That body is not a call of itself.
An operator over `Bool`, `Int`, or `String` is the JVM instruction for it wherever it is written,
which is what `docs/specs/codegen.md` already says, and generic code reaching `Add` through a
constraint is written once per type it is used at and gets the same instruction.
No program can tell which it got, because there is only one thing to get.

The same holds for `Eq`, `Ord`, `Hash`, and `Show` over those three types, and for
`IntegerLiteral<Int>`, whose `from_literal` gives back the whole number it is handed.

What the compiler reads of one of those instances is its head, which says the trait has an
instance for the type.
Version 0.1 does not read the body: nothing calls it, because every use is the instruction.
The body is there because an instance writes one, and because the library is what says in Lumen
what the instruction does.

## The library modules

`prelude` is every name above, and nothing else.

`strings` holds what a `for` loop writes the same way twice.

```text
strings: join
```

`join` runs the parts of a `List<String>` together, with a separator between each pair.

It is written in Lumen, over what the language already gives: a `for` loop, `+`, and `var`.
That is the test a library function is held to.
It lands in the library rather than in the compiler exactly when Lumen can write it, and it lands
at all only when a reader would otherwise write the same loop twice.

Were `strings` to gain a `length`, `list` would have one too, and neither could be a prelude name:
one name has one definition, and a prelude holding both would break that.

A library function may be generic, and `list.length` and `list.has_value` are.
A generic is written once per set of types it is used at, which `docs/specs/codegen.md` states,
so the module declaring it writes the method and the module calling it writes the call.
`has_value` constrains its type parameter by `Eq`, and the body asking for that instance is the
library's, so the instance answering is one the library itself reaches.
Those are the prelude's, which every module has alike, and `L0424` refuses a call that settles
the parameter on anything else.

## What is not here yet

`push`, `split`, and the length of a string each need a JVM method the language cannot yet name.
`docs/implementation.md` section 10's `extern` declaration is what names one, and they land with
it rather than as a compiler-supplied table in the meantime.

The prelude's instance bodies are read by a person rather than by the compiler, so one could say
something other than the instruction it stands for and nothing would notice.
Reading them is a whole pass over the prelude, and it turns up a rule an instance method cannot
keep: `hashed(value: Bool) -> Int` is a `Bool` parameter in a signature that is not all `Bool`,
which `docs/specs/types.md` refuses, and an instance has no say in its own signature.
The rule and the pass land together, and neither lands here.

`map` and `filter` need a parameter whose type is a function, which the grammar of
`docs/specs/grammar.md` does not write.
They also fail the test above twice over: a `for` loop writes each of them in one line, and
`docs/principles.md` question 12 asks what a method earns that a loop does not.

## The errors

| code    | what it refuses                                                      |
| ------- | -------------------------------------------------------------------- |
| `L0306` | an import names a module neither the library nor a file beside holds |
| `L0424` | a call of a constrained library generic, at a type the library reaches no instance of |

A refusal inside the library is the compiler's own failure and not the program's.
The library is compiled with every check a program is compiled with, and a library that does not
compile fails the compiler's own tests before it reaches anyone.

## Properties

These hold and are checked with property-based tests:

1. Every library module is in canonical form, and every one a program may import compiles.
2. A program that writes no import sees every prelude name and no library module's name.
3. The prelude's declarations are the same whichever module asks for them.

The prelude is not among the modules of property 1 that compile on their own.
Its own names are in scope in every module, so a compiler reading it as a module would refuse
every declaration in it as a name already in scope.
It is read against what the JVM holds and nothing else, which is the one scope it fits.

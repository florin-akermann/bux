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

The prelude is compiled before the module a command names, and before every module that one
reaches.
Nothing imports it: its names are in scope in every module, which `docs/specs/modules.md` states.

Another library module is reached by importing it, exactly as `io` is.
`import list` brings `list` into scope under its own name, and `list.length(items)` is a call of
the function `length` that module declares.
An import of a library module looks beside no file: the compiler holds the source, so a file of
that name beside the importing one is not consulted and does not shadow it.

A module a program writes may not be called `prelude`, `list`, or `strings`.
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
functions:    or  todo
traits:       Add  Div  Eq  Hash  IntegerLiteral  Mul  Neg  Ord  Rem  Show  Sub
instances:    each of those traits for the types of it the library writes
```

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
which is what `docs/specs/codegen.md` already says, and the library's instance is the one place
that instruction is wrapped in a method.
Generic code reaching `Add` through a constraint calls that method; everything else is the
instruction, and no program can tell which it got.

The same holds for `Eq`, `Ord`, `Hash`, and `Show` over those three types, and for
`IntegerLiteral<Int>`, whose `from_literal` gives back the whole number it is handed.

## The library modules

`prelude` is every name above, and nothing else.

`list` and `strings` hold what a `for` loop cannot write.

```text
list:    length  contains
strings: join
```

`length` counts what a list holds, and `contains` asks whether it holds a value, which needs
`Eq<T>` and says so.
`join` runs the parts of a `List<String>` together.

Each of the three is written in Lumen, over what the language already gives: a `for` loop, `==`,
and `+`.
That is the test a library function is held to.
It lands in the library rather than in the compiler exactly when Lumen can write it, and it lands
at all only when a reader would otherwise write the same loop twice.

`list.length` and `strings.length` are two names in two modules, which is why neither is in the
prelude: one name has one definition, and a prelude holding both would break that.

## What is not here yet

`push`, `split`, and the length of a string each need a JVM method the language cannot yet name.
`docs/implementation.md` section 10's `extern` declaration is what names one, and they land with
it rather than as a compiler-supplied table in the meantime.

`map` and `filter` need a parameter whose type is a function, which the grammar of
`docs/specs/grammar.md` does not write.
They also fail the test above twice over: a `for` loop writes each of them in one line, and
`docs/principles.md` question 12 asks what a method earns that a loop does not.

## The errors

| code    | what it refuses                                                     |
| ------- | ------------------------------------------------------------------- |
| `L0306` | an import names a module neither the library nor a file beside holds |

A refusal inside the library is the compiler's own failure and not the program's.
The library is compiled with every check a program is compiled with, and a library that does not
compile fails the compiler's own tests before it reaches anyone.

## Properties

These hold and are checked with property-based tests:

1. Every library module compiles, and is in canonical form.
2. A program that writes no import sees every prelude name and no library module's name.
3. The prelude's declarations are the same whichever module asks for them.

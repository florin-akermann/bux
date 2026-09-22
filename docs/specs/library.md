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

Naming a module of a program after a library module therefore buys nothing: the name is taken,
and an import of it is the library's.

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
functions:    or  ok_or
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

Nothing calls one of those bodies, because every use is the instruction, and the compiler reads
each of them all the same.
The library is what says in Lumen what an instruction does, and a claim nothing reads drifts from
what it claims about.
So the prelude is inferred where it is read, which is before the first module of a build is, and
every body in it is held to the type its declaration gives it.

A refusal there is the compiler's own failure and no program's, so it fails the compiler's own
tests rather than reaching anyone.
The prelude is read once however many modules a build has, so a build pays for the walk once.

That walk turns up one rule an instance method cannot keep, and the rule is what gives way.
`hashed(value: Bool) -> Int` is a `Bool` parameter in a signature that is not all `Bool`, which
`docs/specs/arguments.md` refuses as `L0412`.
An instance has no say in its own signature: `trait Hash<T>` wrote it and `instance Hash<Bool>`
settled `T`, so there is no flag the author could have declined to write.
The rule is about the types an author chose, which `docs/design.md` states, so it reaches an
instance method through its trait: `trait Hash<T>` is held to it and `instance Hash<Bool>` is not.
Everything else a module's body is held to, an instance body is held to.

## A list grown and read at an index

`list.push` and `list.at` are the compiler's, and no `extern` declaration names either of them.

`List` is the compiler's, which the section above states, so what a list does is the compiler's
too.
Neither function has a body in `library/list.lm`, because neither can be said in Bux.
A list is built whole by a literal, and no expression the grammar writes builds a list from a
list, so `push` has no body to write.
`at` has one a `for` loop writes, and that body counts to the index and costs what the list holds.

An `extern` is one Java member under a Lumen signature, and neither of the two is one member.
A signature for either writes a type parameter, and `docs/specs/interop.md` refuses one: a type
parameter is carried by nothing a Java descriptor names.
The member that reads a list at an index takes an `int` and throws past the end, and `at` gives
`None` there.
So the two are written out where they are called, the way an operator over `Int` is, and
`docs/specs/codegen.md` states the instructions each one becomes.

```text
fn push<T>(values: List<T>, value: T) -> List<T>
fn at<T>(values: List<T>, index: Int) -> Option<T>
```

`push` gives back the list with `value` after the last element.
The list it was handed is unchanged, because a list is a value and nothing reaches into one.
`at` gives `Some` of the element at `index`.
It gives `None` where the index is below zero, and where it is the length or above it.
Neither is partial, and neither panics.

`at` costs the same at every index, because it reads one slot.
`push` onto the most recent list costs amortized constant time.
`push` onto an older list costs what that list holds.
`docs/specs/codegen.md` states the buffer and the length that give both costs.

## The instances a list has

The prelude writes `Eq`, `Ord`, `Hash`, and `Show` for `List<T>`, each constrained on `T`, and
each body is a `for` loop over the list in `library/prelude.lm`.
`docs/specs/traits.md` states what the four answer, and each one asks `T` for the instance of its
own trait rather than reading an element any other way.

The four are written in the prelude, beside the traits they answer.
An instance belongs in the module that declares its trait or its type, which `docs/design.md`
section 8 states.
`List` is the compiler's type and no module declares it, so the module that declares the trait is
the one place for each of the four.
The instances of a type travel with the type, so every module that holds a list reaches the four.
Each of the four is lowered from its body in `library/prelude.lm`, which `docs/specs/codegen.md`
states.

Each of the four reads its list with `at`, so `push` and `at` are reachable from the prelude as
well as from `list`, and from no other module.
Nothing else changes about either: both stay the compiler's, and both are written out where they
are called.

## The library modules

`prelude` is every name above, and nothing else.

`list`, `strings`, `map`, and `set` each hold what a `for` loop writes the same way twice, and
`io`, `files`, `process`, and `environment` hold what no `for` loop writes at all.
`strings` holds each of them: `join` is the loop, `length` is what no loop reads, and `at` and
`cut` are a check written over two more `extern` declarations.
`list` holds two more, `push` and `at`, which the section above states are the compiler's.

```text
list:        length  has_value  index_of  push  at
strings:     at  cut  join  length
map:         empty  insert  get
set:         empty  insert  has_value
io:          print  println  eprintln
files:       read  write  listed  made  removed
process:     run
environment: read
```

A library module may import another, and `map`, `set`, `files`, and `process` are the four
that do.
Three of them import `list`: `map` grows the children of a node with `list.push` and reads one
with `list.at`, `files` builds the list `files.listed` gives back the same way, and `process`
grows the list a JVM starts a program from.
`set` imports `map`, because a set is the trie a map is, at a key for each value it holds.
Loading hands an imported library module over below the one that imports it, as it does for a
module read out of a file, so nothing about the order a module is read in changes.

`io`, `files`, `process`, and `environment` are written over `extern` declarations, which
`docs/specs/interop.md` states and `docs/specs/io.md` says what each of the four reaches. Each
declares those declarations beside its functions, and every top-level name is public, so every one
of those surfaces is wider than the names above; `docs/specs/io.md` names the rest.
`io.println` writes to standard output and `io.eprintln` writes to standard error, and the two are
the one `io.put_line` over two streams.
`strings` is written over three of them, which is the same again: `length` is one declaration
itself, and `at` and `cut` are a check over the two the section below states.
`strings.length` reaches `String.length`, whose descriptor gives an `int` that the declaration
widens to the `Int` it gives back.
What it counts is what a JVM counts, which is UTF-16 code units: a character the JVM holds as a
pair of units, such as an emoji, counts as two.

`map` and `set` declare the types they are about as well as the functions, and `map` declares
the steps of the walk of the trie beside them, which `docs/specs/collections.md` lists.
One module holds one type, because `empty` has one definition and a module holding both maps and
sets would need two.

`strings.at(text, index)` gives the UTF-16 code unit at `index` as an `Int`.
It gives `None` where `index` is below `0`, or is not below `strings.length(text)`.
`strings.cut(text, from, to)` gives the part of the text from `from` up to but not including `to`.
It gives `None` where `from` is below `0`, where `to` is above the length, or where `from` is
above `to`.
`at` costs constant time, and `cut` costs time linear in the length of the part it gives back.

Each of the two is written in Bux: a bounds check over `strings.length`, and then one `extern`
call.
The one reaches `String.charAt`, declared with a `char` width and a narrowed `index`, and the
other reaches `String.substring`, declared with a narrowed `from` and a narrowed `to`.
Each of the two declarations gives back `Result<Option<T>, String>`: the `Option` is what
`docs/specs/interop.md` asks of a declaration that narrows an argument, and the `Result` is what
keeps the declaration itself total, because every top-level name of a module is public.
Every index counts UTF-16 code units, which is what `strings.length` counts.

`join` runs the parts of a `List<String>` together, with a separator between each pair.

It is written in Bux, over what the language already gives: a `for` loop, `+`, and `var`.
Every function of `map` and `set` is written over the same, with `match` and a declared type
beside them, and no function in the library calls itself.
That is the test a library function is held to.
It lands in the library rather than in the compiler exactly when Lumen can write it, and it lands
at all only when a reader would otherwise write the same loop twice.

`strings` has a `length` and so does `list`, and neither of the two is a prelude name: one name
has one definition, and a prelude holding both would break that.

A library function may be generic, and every function of `list` and `set` is, as is every
function of `map` that a map is written into or read out of.
A generic is written once per set of types it is used at, which `docs/specs/codegen.md` states,
so the module declaring it writes the method and the module calling it writes the call.
`list.has_value` and `list.index_of` each constrain the type parameter by `Eq`, and the method
written for one set of types calls the instance the type in that set has, wherever it is declared.
`map` and `set` constrain a key by `Eq` and by `Hash`, which `docs/specs/traits.md` states is
written with `+` between the two, and the method reaches an instance of each.
`docs/specs/codegen.md` states how: the library names the program's instance by the module the
type carries in front of its name, so a program's own `Eq` answers a constraint the library wrote.
The trait is the prelude's, which is what has both modules name it; `L0424` refuses a constraint
written over a trait that stays in the module declaring the generic.

## What is not here yet

`split` needs a JVM method whose descriptor names an array of `String`, which no `extern`
declaration can name.
`docs/specs/interop.md` states what crosses, and it lands with whatever names such a member.

A mapping and a filtering over a list need a parameter whose type is a function, which the
grammar of `docs/specs/grammar.md` does not write.
They also fail the test above twice over: a `for` loop writes each of them in one line, and
`docs/principles.md` question 12 asks what a method earns that a loop does not.
The module called `map` is the one that holds `Map<K, V>`, and is neither of them.

A map's size, a removal from one, and a literal for either type wait for the same test, which
`docs/specs/collections.md` says of each of them.
A program that cannot be written without one is what lands it.

## The errors

| code    | what it refuses                                                      |
| ------- | -------------------------------------------------------------------- |
| `L0306` | an import names a module neither the library, a file beside, nor a package holds |
| `L0418` | a call of a constrained library generic, at a type with no instance of that trait |

A refusal inside the library is the compiler's own failure and not the program's.
The library is compiled with every check a program is compiled with, and a library that does not
compile fails the compiler's own tests before it reaches anyone.

## Properties

These hold and are checked with property-based tests:

1. Every library module is in canonical form, and every one a program may import compiles.
2. A program that writes no import sees every prelude name and no library module's name.
3. The prelude's declarations are the same whichever module asks for them.
4. Every instance body the prelude writes has the type its trait gives it at the instance's type.
5. `list.at` gives `Some` of the element at every index a list holds, and `None` at every other.
6. `list.push` gives back what the list held, with the value after it, and leaves the list alone.

The prelude is not among the modules of property 1 that compile on their own.
Its own names are in scope in every module, so a compiler reading it as a module would refuse
every declaration in it as a name already in scope.
It is read against what the JVM holds and nothing else, which is the one scope it fits.

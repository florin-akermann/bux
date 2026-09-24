# Bux

> Go's simplicity.
> Haskell's types.
> Erlang's messages.
> Valhalla's values.
> A compiler written in Bux, which compiles itself.

Bux is a small, statically typed language for practical software, built on Valhalla.
Its compiler emits JVM bytecode, and it is written in Bux.

This is an experiment and a fun project, not a product.
Nothing is stable, nobody supports it, and the language changes whenever a better answer appears.
It exists to find out how much a language gets from very little.

## The name

The language is Bux, the command is `bux`, and a source file ends in `.bx`.
`bin/bux` starts it, and every command below is written as it runs today.

## Philosophy

The goal is not the most powerful language.
The goal is the most useful guarantees for each unit of language complexity.
`docs/principles.md` holds the twelve questions that a proposed feature must answer before it lands.

### One way to write a thing

Sugar is a second spelling, and a second spelling is a cost with no guarantee behind it.
A reader learns both, a formatter chooses between them, and each later feature answers to two forms.
`++`, `--`, `-=`, `*=`, `/=`, `%=`, and a ternary `?:` are refused, and they stay refused.
`+=` is the one shorthand there is, and it is the ceiling rather than the first of a set.

### One way to run work at the same time

Concurrency is Erlang's: a process owns its state, and a message is the only way to that state.
Bux takes that one idea, and it takes nothing else Erlang has.
There is no `async`, no `await`, and no second colour of function.

Every process is written the same way, and the compiler gives it no second shape.
A `process` declares `start`, which builds the first state, and `receive`, which takes one message.
The body of `receive` is one `match`, and each arm is one call, so a branch must name what it does.
Seen one process, seen them all.

Bux has no channel, because a queue that belongs to no process is shared state with a name.
An in-application message bus is a process that holds the handles of whoever cares.
A program writes that bus in Bux, because the language gives processes and messages and no more.
Erlang's crash, supervisor, link, monitor, and unbounded mailbox are each refused.
Section 15 of `docs/design.md` says why each one goes.
Concurrency is in: Item 091 landed `process`, `spawn`, and `send`, and the test runner runs on them.

### Nothing panics, ever

No operation is partial, so `17 / 0` is `None`, and there is no `unwrap`.
An operation with no answer for some of its input says so in its type, never at run time.
The compiler answers to the same rule: a program it cannot compile gets a diagnostic, not a crash.

### No special cases

A rule holds for every type, function, and operator alike, or it is no rule.
`Int` is a type like any other, and a library could declare everything the prelude supplies.
An operator is a function with other syntax, and `main` is a function like any other.
What `Int` does, a type that a program declares does, and `Int` does nothing more than it.

Java shows what the other answer costs.
Its `int` is not an object, so a generic holds an `Integer`, and the two types are not the same.
`List<int>` does not compile.
`==` compares values for `int`, but it compares identity for `Integer`.
A cache of small numbers hides that difference until a program uses a large number.

The library then writes some abstractions two times.
`Stream` has `IntStream` beside it, and `Optional` has `OptionalInt` beside it.
A caller picks `comparing` or `comparingInt`, and each new feature answers to the two halves.
The split starts at one type and reaches the generics, the operators, and the library alike.
Bux keeps one half, so a user of the language declares a type that is `Int`'s equal in every way.

### Values, not objects

No type a program declares has identity, a `hashCode`, or a `toString`.
Equality is opt-in: `==` needs `Eq`, a type derives `Eq`, and it compares what the value holds.
Every type is a Valhalla value class from the first day: identity-free, null-free, equal by state.

### The JVM is a target, not a model

Boxing and the primitive/reference split stay inside the compiler, where no program can see them.
Java's object model, null, checked exceptions, and inheritance stay out.

### A small library, and a `for` loop

The fewer methods a type has, the better.
A method lands only where a plain loop cannot write it, which is why a map has no iterator.
Higher-order functions are a library, never a second way to write a program.

### Canonical form, or it does not compile

Source that is not in canonical form is a compile error, and `bin/bux fmt` writes that form.
There is no style option, and a review has nothing about layout to argue about.

### Every function has a name

There are no anonymous functions, and a function is a first-class value by its name.

## Status

Version 0.1 and version 0.2 are in: types, records, ADTs, `match`, generics, and inference.
Traits, `derive`, operators as trait methods, collections, modules, and packages are in with them.
Version 0.3 is self-hosting, and Item 075 completed it: the compiler in `compiler/` builds itself.
Item 087 then deleted the Rust compiler that was the bootstrap.
A seed, `bin/seed.jar`, builds the compiler now, and the compiler then builds itself again.
The runner under `tests/` holds the compiler to every example and every drawn property.
Version 0.4 adds a native binary, HTTP, and JSON, and concurrency is in already.
`TODOS.md` is the backlog, and it is the only task tracker this repository has.

## Documents

- `docs/design.md` is the language specification; a language change is a change there first.
- `docs/implementation.md` says how the compiler is built and what ships in which version.
- `docs/principles.md` holds the questions that every proposed feature must answer.
- `docs/specs/` holds a behaviour spec for each feature, written before the feature lands.
- `tests/spec/` holds executable examples, and they are the specification a reader can run.
- `AGENTS.md` holds the rules that each contributor, human or agent, works to.

Most of the code is written by AI agents that work to `AGENTS.md`, which is why that file is strict.

## Prerequisites

- JDK 28 or later, which runs the compiler and every program it compiles
- Until JDK 28 ships, an early-access build of it; export `JAVA_HOME` as its `Contents/Home`
No Cargo and no other build tool is needed.
`mycs` is optional: the hook runs its sweep where it is on the `PATH`, and skips it where it is not.

## Build

```sh
bin/bootstrap
bin/runner
git config core.hooksPath .githooks
```

`bin/bootstrap` builds the compiler from the seed, and then the compiler builds itself again.
The two builds must be the same, byte for byte.
`bin/runner` holds the compiler to every example, every example line, and every drawn property.
`bin/bux` starts the compiler; a link to it on the `PATH` works like any other installed compiler.

## The example program

`example/main.bx` is everyday Bux in one screen: a record, an ADT, a `match`, and a list walked.

```sh
bin/bux run example/main.bx
```

It writes each answer it works out and exits `0`:

```text
held: 7
each: 3
most: 6
```

`docs/specs/example-program.md` says what the directory holds and why that is enough.

Its public surface is one page, which is what a reader consults to learn a signature:

```sh
bin/bux api example/main.bx
```

`bin/bux --help` lists every command: `fmt`, `check`, `build`, `run`, `test`, `api`, and `explain`.

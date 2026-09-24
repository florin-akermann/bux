# Bux: Design Rationale

This file gives the reasons behind the rules of `docs/design.md`.
Each section has the number and the heading of the `docs/design.md` section whose rules it argues.
Read it to propose a change to a rule; to write Bux, `docs/design.md` is enough.
A reason here never adds a rule: a sentence that states what the language does belongs there.

Bux inherits neither Java's object model, Rust's ownership model, nor Haskell's complexity.

---

## 1. Goals

### Erlang

Bux takes one idea from Erlang and refuses the rest, and section 15 gives the reason for each.

### Valhalla

Valhalla's data model is the one part of the JVM Bux adopts whole, because it is Bux's own.

### Rust

The first compiler was written in Rust, to benefit from:

* strong compiler implementation safety
* excellent tooling
* algebraic data types
* exhaustive matching
* fast compilation
* good performance
* mature ecosystem

Rust was the bootstrap, and the compiler is now written in Bux.

---

## 2. Non-goals

The guiding principle is:

> Strong static guarantees, without the programmer understanding the runtime's memory model.

A crash, a supervisor, and a restart are each refused, and section 15 says why Bux needs none.

### Sugar is a second spelling, and a second spelling is a cost

A shorthand that writes what the language already writes buys nothing the type system can check.
A reader learns both spellings, and canonical form has to choose between them.
Every later feature then answers to two forms rather than one.
That is why the sugar of design.md section 2 is refused rather than deferred.
`a = a + 1` and an `if` say each of them in the one shape the rest of the language has.
`+=` is kept because a `for` loop that totals is the everyday shape Bux is built around.

### Nothing panics

Rust panics on `x / 0` and calls the panic a design; Bux does not, as a crash is an untyped answer.

---

## 3. Core design

A language with several whole number types asks every author to choose a width that rarely matters.

---

## 4. Algebraic data types

A guard is refused because an `if` inside the arm says the same thing.
A guarded arm is also an arm no exhaustiveness check can reason about.

---

## 5. Option and Result

Rust panics on `x / 0` and calls the panic a design; Bux refuses the trade, and the type answers.
So a divisor is never a hazard a reader has to spot, and arithmetic through `?` reads as arithmetic.

Where the absence is the whole story, an error type would carry nothing the caller is not holding.
That is why such a case reaches for `Option` rather than `Result`.

`Some(())` says only that a value is there, and `None` that it is not: `Bool` spelled a second way.
At the platform boundary it is worse, because `Some` there also reads a `null` that did not come.
`Some(())` is then a flag for `null`, and nullability is a non-goal of section 2.
`Result<(), E>` stays, because its `Err` carries a reason the caller could not work out.

`or(maybe, fallback)` is named for what it does rather than for what it is not.

A value nothing takes is a compile error, because a dropped `Result` is a swallowed failure.

A hole makes incompleteness greppable and gated, rather than filled in with plausible wrong code.

---

## 6. Type inference

A signature is a boundary: a caller and the `bux api` page read it, and neither reads the body.
That is why every function states its types, and a binding inside a body does not.

---

## 7. Generics

### A generic is specialized at each use

The alternative is erasure, which compiles one body over a type that every value fits.
It pays for that by boxing every `Int` on the way in and casting it back on the way out.
That would make `Int` the one type a program can tell from a declared one.
Section 3 says there is no such type.

---

## 8. Typeclasses / traits

The design takes its typeclasses from Haskell and from Rust traits.

One type parameter is what `Eq<T>`, `Ord<T>`, and `IntegerLiteral<T>` each ask for.
It is all the language has needed.

`==` was only the first operator to be written as a trait method.
The wiring version 0.1 shipped named `Int` and `String` where an instance now stands.
It was the degenerate case of this design rather than a design of its own, and it is gone.

---

## 9. Records

This section has no rationale beyond its rules.

---

## 10. Immutability

This section has no rationale beyond its rules.

---

## 11. Functions

A name forces the author to say what the function is for, and gives the reader a word to search.
A function name that is not called is refused rather than lowered.
So nothing reaches code generation that it has no way to write.

### A function is called one way

A second spelling of one call is a cost with no guarantee behind it, as section 2 states.

### Named arguments

Where two parameters share one type, nothing else tells them apart.
`rename(old, new)` and `rename(new, old)` both typecheck, and one of them is wrong.
A mistake a type system can make unwriteable belongs in the language rather than in a linter.

### A parameter is never a bare `Bool`

`open(true)` says nothing.
The reader has to find the declaration to learn what is true.
`open(false)` is one keystroke away from a program that does the other thing without looking wrong.
`open(path, ReadOnly)` reads where `open(path, true)` did not.
A third mode is then a variant rather than a second flag.

A function whose parameters and result are all `Bool` is about `Bool`, rather than told one.
So its operands stay writable.

A call is the one place a bare `true` loses its meaning, so the rule is about what a call passes.

An instance has no flag its author could have declined to write.
The two-variant type the rule asks for is one the trait would have had to take.
A trait that writes a `Bool` parameter is refused, because every instance would then take that flag.
Nothing is given up: the one place such a parameter can be written is the one place it is read.

### The entry point

A program already knows what it is, so its name is not one of its arguments.

### Every function carries an example

A signature says what a function takes and gives back, and says nothing about what it does.
Prose says that and drifts, because nothing runs prose.
An example says it so the compiler can hold the function to it, and a stale one is a failing test.

### A test states what one line cannot

An example is one expression on one comment line.
A claim that binds names, builds a value in a loop, or calls `io` does not fit on that line.
Without a test block, such a claim becomes a function written only to be called by an example.
That function would then need an example of its own, and it would ship in every program.

---

## 12. Control flow

This section has no rationale beyond its rules.
`docs/principles.md` question 12 says why a function that takes a function is a loop written twice.

---

## 13. Formatting

### Naming

A one-character name such as `f` names nothing a reader can look for.
`if is_active(user)` reads as a question where `active(user)` reads as a command.
A name is the author's decision, so the compiler checks a name and never rewrites it.

### Order

Whitespace is nobody's decision, so `bux fmt` repairs it.
The place of a declaration is the author's decision, so the compiler says where and never moves it.

---

## 14. Effects

### Resources are the same feature

Go's `defer` and Java's try-with-resources give only the first of the two guarantees.
A closed handle can still escape.
The second guarantee needs the type system, and the classic answer is linear types.
Linear types are ownership's family, which section 2 refuses, so the answer is an escape check.

A foreign reference is held to the escape check because of the object behind it.
That object has identity and mutates, so one a process holds is shared state again.
So the escape check does one job, rather than a second rule written for the boundary.

Elsewhere most of the cost of such a check is closures, which capture capabilities silently.
Bux has no anonymous functions, and a named function cannot capture a local.
The escape routes are therefore enumerable, and the rule stays one paragraph.
That is a standing reason to keep the no-closures rule when it feels inconvenient.

`AutoCloseable` never surfaces because it is a JVM interface, and the JVM is a target, not a model.

---

## 15. Concurrency

Bux has one kind of function, and no caller ever has to ask which kind it holds.
A target that makes blocking cheap needs no second kind.

Seen one process, seen them all: a reader knows where the state starts and where each message goes.
`Next<T>` is an ordinary ADT, so `docs/principles.md` question 8 is answered.

The rule that an arm of `receive` is one call or one name keeps a process readable as it grows.
A branch cannot hold the work, so the author must lift the work out and give it a name.
The compiler holds the shape, and it does not judge the name it forced the author to write.
A rule about a good name is not a rule a compiler can hold, and Bux does not pretend otherwise.

The shape is a declaration because of `docs/principles.md` question 7.
Elegance leaves the invalid case unwriteable.
A hand-written loop lets every author write a slightly different process.
A linter over that loop rejects each one after the fact, which question 7 calls adequacy.

Question 12 holds a library function to a `for` loop, and a `process` declaration is no function.
Read broadly, the question still lands: a `for` loop could write this, and that is the problem.
A process is the one place where a second shape costs a reader the whole program.
A second form would be the second spelling that question 9 refuses.

A lock protects shared mutable state, and no type a program declares holds any.
A record is rebuilt rather than reached into, as section 10 says.
Section 14 lists mutable global state as an effect for the same reason.
A foreign reference is no value a program declared, and the escape check keeps it out of a process.

Go gives a queue an identity of its own, and any process may hold it.
Such a queue is shared mutable state with a name, which the value model refuses everywhere else.
It also needs rules that a mailbox does not need.
Nothing owns a channel, so closing one is a protocol rather than an event.
A reader holds several channels, so a `select` must choose among them.
One queue with many readers is the one shape a channel has and a mailbox does not.
A worker pool is that shape, and a process answers it.

Erlang lets a mailbox grow without a bound, and one slow process then exhausts the memory.
Bux refuses that, because the type system cannot see it and the program cannot recover from it.
A process that owns shared state is slower than a lock on a hot path, and the trade is accepted.

Erlang drops a message sent to a process that has ended, and it never tells the sender.
The rule that gives every operation an answer also refuses Erlang's crash.
A supervisor, a link, and a monitor each answer a failure that Bux's types answer first.

Erlang's selective receive walks the mailbox for a message that matches and leaves the rest behind.
It reads well, and it costs one scan for each receive, which a full mailbox makes slow.
A `send` that says how long it waits is what a bus that drops the oldest message needs.
No other primitive answers that need.

A `send` names the process it reaches, and a program often has a message that whoever cares reads.
A bus is the process that closes that gap.
A bus is made of processes, and that is why the process is underneath and the bus on top.

Erlang makes a remote process look like a local one, and Bux does not.
A network call fails where a local `send` does not, and one name for both hides that difference.

---

## 16. Modules and names

A module is one file and a file is small, so a private declaration waits for a case that needs one.

A reader who meets `io.print` knows where to look without knowing what else the file imports.
An unqualified import would take that away.

With one definition for each name, a search for a name finds its definition and its uses.
Nothing else is mixed in.

---

## 17. Reaching the platform

A platform has libraries that are worth reaching, and the model that comes with them is not.

Reflection, the loading of code, handles to members, and deserialization are left off the list.
Each of them reaches code that the program text does not name, which no reader can check.

No subtyping and no implicit conversion keep the boundary a signature, not a second type system.

A foreign reference a process holds is shared mutable state, which section 15 has no answer for.
The escape check of section 14 is that answer, rather than a rule of the boundary's own.

The claim of what an `extern` can fail with is the one claim nothing checks.
That is why the boundary is narrow and lives in the library.

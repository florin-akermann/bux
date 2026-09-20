# Lumen: Language Design

> Go's simplicity.
> Haskell's type system.
> Valhalla's values.
> A compiler written in Rust.

A small, statically typed language for building practical software.
It is built on Valhalla, the JVM's value classes, and every Lumen value is a value.
Nothing has identity, and equality is by state, only where a type asks for it.
No built-in type is special: a type a library declares can do everything `Int` can.
It inherits neither Java's object model, Rust's ownership model, nor Haskell's complexity.

The compiler is written in Rust and targets JVM bytecode, on JDK 28 or later.
The JVM is the first compilation target and nothing more; section 2 says what that rules out.

This document is the language specification; every change to the language is a change here first.
How the compiler is built and what ships when is in `docs/implementation.md`.
The questions every proposed feature must answer are in `docs/principles.md`.

---

## 1. Goals

The language combines five properties.

### Go

Take inspiration from Go's:

* small language surface
* readable syntax
* fast compilation
* straightforward tooling
* simple deployment model
* practical standard library
* built-in concurrency
* preference for explicit, boring code
* one canonical formatting

### Haskell / ML

Take inspiration from the ML/Haskell family for:

* algebraic data types
* exhaustive pattern matching
* type inference
* parametric polymorphism
* typeclasses
* immutable data
* `Option`
* `Result`
* eventually, explicit effects

### JVM

Use the JVM for:

* garbage collection
* JIT compilation
* threads
* virtual threads
* networking
* filesystem APIs
* cryptography
* mature profiling/debugging
* cross-platform execution
* the existing Java ecosystem

The JVM is an implementation target, not the semantic model of the language.

### Valhalla

Build every type on the JVM's value classes, so that:

* a value has no identity, and two built alike are one value
* equality is by state, and only where a type says it is
* a record or a variant is laid out flat wherever the JVM can flatten one
* a built-in type and a declared type are the same kind of thing
* boxing is the compiler's business and never a program's

Valhalla's data model is the one part of the JVM Lumen adopts whole, because it is Lumen's own.
Every class the compiler writes is a value class, from the first release on.

### Rust

Write the compiler in Rust to benefit from:

* strong compiler implementation safety
* excellent tooling
* algebraic data types
* exhaustive matching
* fast compilation
* good performance
* mature ecosystem

---

## 2. Non-goals

The language should **not** attempt to be:

* Rust without lifetimes
* Haskell with Go syntax
* a better Java
* a Scala replacement
* a JVM version of C#
* a language with every advanced type-system feature imaginable

In particular, the language should initially avoid:

* ownership
* borrowing
* lifetimes
* manual memory management
* inheritance
* class hierarchies
* null
* exceptions, checked or otherwise
* macros
* complicated metaprogramming
* implicit runtime magic
* excessive syntax
* anonymous functions
* async/await, or any other function colouring
* identity, or an equality every type has whether or not it asked for one
* boxing a program can observe, or a built-in type that is special

### The JVM is a target, not a model

The JVM is where Lumen compiles first, and that is the whole of its authority over the language.
None of its constraints is inherited.
Not the object model: no identity, no `equals` on everything, no `hashCode`, no root class.
Not the eight primitive types that are special against every other; `Int` is a type like `User`.
Not boxing, which a program never observes, and not erasure, which is why a generic boxes.
What Lumen adopts instead is value semantics, in the shape Valhalla gives a value class.
That shape is no identity, no null, and equality by state, and every Lumen type already has it.
`docs/principles.md` asks of every feature whether the JVM leaks through it; this is the rule.

The guiding principle is:

> Strong static guarantees, without the programmer understanding the runtime's memory model.

---

## 3. Core design

A program should look approximately like this:

```text
type UserId = UserId(Int)

type Email = Email(String)

type User = {
    id: UserId
    name: String
    email: Email
}

type FindUserError =
    | NotFound
    | DatabaseError(String)

fn find_user(id: UserId) -> Result<User, FindUserError> {
    ...
}
```

The compiler should distinguish:

```text
UserId
```

from:

```text
Int
```

even though they may have essentially the same runtime representation.

Likewise:

```text
Email
```

must not be interchangeable with:

```text
String
```

without an explicit conversion.

This makes domain concepts first-class types.

`Int` is the one whole number type the prelude supplies, and it is 64 bits wide.
A language with several of them asks every author to choose a width that almost never matters.
A program that needs another width declares one, and the type it declares is no lesser than `Int`.

That last sentence is a promise the language is held to, and it has three parts.
A declared type can be written as a literal, so `let x: Int32 = 1` is as plain as `let x = 1`.
A declared type can own an operator, so `a + b` over two `Int32`s is what its library says it is.
A declared type has `Eq` when it says so, and `Int` has `Eq` for the same reason and no other.
Section 8 gives the mechanism for all three: an operator is a trait method, and so is a literal.
`Int`, `Bool`, and `String` reach the operators through library instances, exactly as `Int32` does.
A capability given to a prelude type and withheld from a declared type is a leak.
`docs/principles.md` asks every feature whether it opens one.

---

## 4. Algebraic data types

Algebraic data types are a fundamental language feature.

```text
type Payment =
    | Pending
    | Authorized {
        authorization_id: String
    }
    | Captured {
        transaction_id: String
    }
    | Failed {
        reason: String
    }
```

Pattern matching:

```text
fn describe(payment: Payment) -> String {
    match payment {
        Pending =>
            "Waiting"

        Authorized { authorization_id } =>
            "Authorized: " + authorization_id

        Captured { transaction_id } =>
            "Captured: " + transaction_id

        Failed { reason } =>
            "Failed: " + reason
    }
}
```

The compiler must verify that matches are exhaustive.

Adding a variant must therefore make the compiler point at every affected match expression.

---

## 5. Option and Result

Null should not exist in ordinary language code.

Optional values use:

```text
Option<T>
```

For example:

```text
fn find_user(id: UserId) -> Option<User>
```

Failure uses:

```text
Result<T, E>
```

For example:

```text
fn save_user(user: User) -> Result<(), DatabaseError>
```

Both should be ordinary algebraic data types rather than special magic wherever practical.

Error propagation should be concise:

```text
fn load_user(id: UserId) -> Result<User, Error> {
    user := find_user(id)?
    validate(user)?
    Ok(user)
}
```

The `?` operator propagates the case that has nothing to go on with.
On a `Result` it hands the `Err` back, and on an `Option` it hands the `None` back.
Each lands in a function that gives back the same kind, so `?` has nothing to convert.
A `None` met where the function gives back a `Result` is refused, because it names no error.

**No program throws, catches, or observes an exception, and no operation is partial**.
An operation without an answer for some of its input says so in its type rather than at runtime.
That holds for an operator as much as for a function.
An operator is a function with other syntax, and syntax buys no exemption from the type.

A module the compiler supplies may sit on a JVM operation that throws, and gives back a `Result`.
The throw is caught where the module is built, and never reaches the program.
`docs/specs/io.md` states it for the one read version 0.1 has.

So every operator that can fail gives back an `Option` or a `Result`, and never a bare answer.
`/` and `%` are the ones version 0.1 has, and `17 / 0` is `None` rather than a crash.
A divisor is never a hazard a reader has to spot.
An operator with an answer for every input keeps its plain type, which is why `Int + Int` is `Int`.
Wrapping on overflow is a defined answer, and there is no whole number equal to `x / 0`.
An operator is a trait method, and section 8 says so; the rule binds every instance alike.
A type whose `/` has an answer for every divisor may give a plain result, and `Int`'s does not.
Arithmetic over `Int` divides through `?`, one per division, and reads as arithmetic.
`or` is for the author who means a fallback, and `match` for the zero divisor with something to say.

Which of the two an operation reaches for is settled by what the failure has to say.
`Option` is for the case that explains itself, where the absence is the whole story.
An error type there would carry nothing the caller is not already holding.
`Result` is for the failure with something to say that the caller could not work out.
`?` propagates either, each into a function that gives back its kind.
`docs/specs/arithmetic.md` works the choice through for `/` and `%`.

There is no `unwrap` and no `expect`, in the prelude or anywhere else.
`or(maybe, fallback)` is the total default, named for what it does rather than for what it is not.

**A value nothing takes is a compile error**, because a dropped `Result` is a swallowed failure.
A statement written for its effect has nothing to leave behind, so its type is `()`.
Throwing a value away is written rather than implied: `_ = save(user)` says it and `_` is no name.
`docs/specs/discarding.md` states which statements give their value away and which discard it.

**An unfinished body says so in the language**, with `todo("a reason")` where a value belongs.
A hole takes whatever type is expected of it, so the work around it is typed like finished work.
`lumen check` accepts a hole and `lumen build` refuses every one it finds, naming each.
Incompleteness is then greppable and gated, rather than filled in with plausible wrong code.
`docs/specs/holes.md` states what the two commands do.

---

## 6. Type inference

Type annotations should be required at public boundaries when useful.
Local code should rely heavily on inference.

For example:

```text
fn identity(x) {
    x
}
```

should infer approximately:

```text
∀T. T -> T
```

Likewise:

```text
x := 42
name := "Florin"
```

should infer:

```text
x: Int
name: String
```

The language should aim for Haskell/ML-level inference while maintaining Go-like readability.

---

## 7. Generics

Generic programming should be fundamental.

```text
fn first<T>(items: List<T>) -> Option<T> {
    ...
}
```

Generic data types:

```text
type Result<T, E> =
    | Ok(T)
    | Err(E)
```

The type system should support parametric polymorphism without requiring verbose annotations.

### A generic is specialized at each use

**A generic function is compiled once for each set of types it is used at**, and never once for
all of them.

`identity(1)` and `identity(word)` reach two different methods: one taking a whole number, one
taking text.
Neither boxes, neither casts, and an `Int` crossing a generic is the same 64 bits it is anywhere
else.

The alternative is erasure, which compiles one body over a type that every value fits, and pays
for it by boxing every `Int` on the way in and casting it back on the way out.
That would make `Int` the one type a program can tell from a declared one, and section 3 says
there is no such type.

A use that settles no type settles none for the machine either, and two uses that differ only in
a type the machine cannot tell apart are one use.
`docs/specs/codegen.md` states what is written and what it is named.

---

## 8. Typeclasses / traits

The language should provide a simple form of typeclasses inspired by Haskell and Rust traits.

```text
trait Eq<T> {
    fn equals(a: T, b: T) -> Bool
}
```

Generic functions can constrain their types:

```text
fn contains<T: Eq<T>>(
    items: List<T>,
    value: T
) -> Bool {
    ...
}
```

`==` is `Eq`: a type is compared only when it has an instance, and a built-in type is no exception.
Version 0.1 has no `derive`, so the library ships the three instances: `Int`, `Bool`, and `String`.
`==` on a record or a variant is refused until its type derives `Eq`, which version 0.2 allows.

Every operator is a trait method, and `==` is only the first to be written that way.
`+` is `Add`, `-` is `Sub`, `*` is `Mul`, `/` is `Div`, `%` is `Rem`, and prefix `-` is `Neg`.
`<`, `<=`, `>`, and `>=` are `Ord`, and `&&`, `||`, and `!` stay `Bool`'s alone: they short-circuit.
An operator's type is its instance's type: `Div<Int>` gives `Option<Int>`, `Add<Int>` an `Int`.
The library ships the instances for `Int` and `String`; a declared type writes its own the same way.
A type without an instance has no operator, so `a + b` over two `UserId`s is refused, as it is now.
Version 0.1 wires `Int` and `String` to the operators directly, because it has no typeclasses.
That wiring is the degenerate case of this design, not a design of its own, and 0.2 replaces it.

A literal is a trait method too, so a declared type can be written as plainly as `Int` can.
A whole-number literal takes the type the context expects, provided that type has `IntegerLiteral`.
A literal that does not fit its type is a compile error where it is written.
It is never a wrapped value and never a runtime failure, because a literal is no exception.
A literal whose type nothing settles is an `Int`, which is the one default the language keeps.
The spec that lands the trait settles how an instance states what fits.

Standard traits should include concepts such as:

```text
Eq
Ord
Hash
Show
```

Common implementations should be derivable:

```text
derive Eq, Ord, Hash for User
```

Advanced type-level machinery should not be introduced until there is a concrete need for it.

---

## 9. Records

Records should be lightweight and pleasant to use.

A record is a value with no identity: nothing can ask whether two of them are one object.
Two records holding the same fields are one value, which is what a Valhalla value class makes true.

```text
type User = {
    id: UserId
    name: String
    email: Email
}
```

Record construction:

```text
user := User {
    id: id,
    name: "Alice",
    email: email
}
```

Record updates should be concise:

```text
updated := user {
    name: "Bob"
}
```

The language may eventually support structural anonymous records:

```text
fn contact(user: User) {
    {
        name: user.name,
        email: user.email
    }
}

result := map(users, contact)
```

This should be considered only after the core type system is stable.

---

## 10. Immutability

Data should be immutable by default.

Mutation should be explicit.

Prefer:

```text
user := user {
    name: "Bob"
}
```

over implicit mutation.

If mutable state is required:

```text
var counter = 0

counter += 1
```

The distinction should be obvious in source code.

**An assignment names a name**.
`counter = 1` and `counter += 1` are the whole of it; `user.name = "Bob"` is not written.
A record is updated by building the value it becomes, which the paragraph above shows.
There is then one way to change what a name holds, and none to reach inside a value.

**The name an assignment names is a `var` binding**.
`total := 0` promises the reader that `total` never changes, so `total = 2` below it is refused.
A parameter, a `for … in` binding, and a name a pattern binds never change either.
Mutation is explicit, which means it is visible at the binding rather than only at the change.

---

## 11. Functions

Functions are first-class values.

```text
fn add(a: Int, b: Int) -> Int {
    a + b
}
```

A function is a value by its name, so it can be stored, passed, and returned:

```text
fn is_active(user: User) -> Bool {
    user.active
}

fn user_name(user: User) -> String {
    user.name
}

active_names := map(filter(users, is_active), user_name)
```

**There are no anonymous functions**.
Every function has a name, and the name is written where the function is used.
Nested named functions may be declared inside a function body when they are local to it.

**Version 0.1 reaches a function by calling it, and no other way**.
The snippet above is where the language is going, not what version 0.1 compiles.
Passing `is_active` to `filter` waits on a function value having a type and a shape.
A function name written as anything but the name of a call is refused rather than lowered.
Nothing then reaches code generation that it has no way to write.

A name forces the author to say what the function is for, and gives the reader a word to search.

### Named arguments

A call passes its arguments in order.
Where the order is the only thing holding them apart, the call writes the parameter names too.

```text
rename(from: old, to: new)
```

A name is written before its value and joined by `:`, exactly as a record writes a field.
The names run in the order the declaration lists the parameters.
Naming reorders nothing; it says what the order already is.
A call names all of its arguments or none of them.

A call must name them when the declaration gives two of its parameters one type.
`add(a: Int, b: Int)` above is one, so `add(1, 2)` is refused and `add(a: 1, b: 2)` is not.
Nothing else tells two of one type apart.
`rename(old, new)` and `rename(new, old)` both typecheck, and one of them is wrong.
A mistake a type system can make unwriteable belongs in the language rather than in a linter.

The types compared are the ones inference settled: an unwritten signature counts as a written one.
A type parameter counts as a type.
A constructor carries its values in order and has no names to write, so naming them is refused.
A variant whose values want names declares them as fields and is built as a record.

`docs/specs/arguments.md` is the specification.

### A parameter is never a bare `Bool`

`open(true)` says nothing.
The reader has to find the declaration to learn what is true, and `open(false)` is one keystroke
away from a program that does the other thing without looking wrong.

A parameter of type `Bool` is refused.
A two-variant type takes its place, and the call then says which of the two it means:

```text
type Mode =
    | ReadOnly
    | ReadWrite

fn open(path: String, mode: Mode) -> File {
```

`open(path, ReadOnly)` reads where `open(path, true)` did not, and a third mode is a variant
rather than a second flag.

A function whose parameters are all `Bool` and whose result is `Bool` is the one carve-out.
`Bool` is what such a function is about, rather than something it is told, so its operands stay
writable.

A record field, a variant payload, a binding, and a result type may each be `Bool`.
The rule is about what a call passes, which is the one place a bare `true` loses its meaning.

### The entry point

A program starts at `main`:

```text
fn main() -> () {
    greet("world")
}
```

`main` takes nothing and gives back nothing: a program is run for what it does.
A module declaring it can be run; one that does not is a library, and running it is refused.

A run is over when `main` is.
There is no exit status to write, because a program has nothing to say yet about how it went.

### Every function carries an example

**A function a module declares at the top level states at least one example**, or it is not built.

```text
// Divides `total` among `people`, giving back nothing where there is nobody to divide among.
//
// example: or(shared(total: 17, people: 5), 0) == 3
fn shared(total: Int, people: Int) -> Option<Int> {
    total / people
}
```

An example is a line of the comment above the function, and it is Lumen rather than prose.
It is an expression of type `Bool`, and `lumen test` runs every one a module states.

A signature says what a function takes and gives back, and says nothing about what it does.
Prose says that and drifts, because nothing runs prose.
An example says it so the compiler can hold the function to it, and a stale one is a failing test.

`main` is exempt: it is reached by running the module, so running the module is its example.
`lumen check` accepts a function that states none, because a function is written before the
example over it can compile.
`docs/specs/doc-examples.md` is the specification.

---

## 12. Control flow

Everyday code is imperative and reads like Go: basically a bunch of `for` loops.

```text
fn active_names(users: List<User>) -> List<String> {
    var names = List.empty()
    for user in users {
        if user.active {
            names = names.push(user.name)
        }
    }
    names
}
```

A list is written between brackets: `[first, second]`, and `[]` holds nothing.
That is the language's own way to build a list, and it builds it whole.
`List.empty` and `push` above are library code that a later version supplies.

The control-flow surface is:

* `if` / `else`
* `for item in collection`
* `for condition` and a bare `for` (an infinite loop)
* `break` and `continue`
* `match`
* `return`

Plain loops are the default idiom.
Higher-order functions such as `map` and `filter` are library code, not a second programming model.
A pipeline operator (`|>`) is not part of the language until a concrete need for it is shown.

---

## 13. Formatting

There is exactly one canonical formatting of every program.

**Source that is not in canonical form does not compile**.
The compiler reports the first deviation as an error, in the same voice as any other diagnostic.
`lumen fmt` rewrites a file into canonical form; `lumen build` refuses a file that is not in it.

The formatter is therefore part of the compiler front end, and the pretty-printer is its definition.
Compilation requires `format(source) == source`, byte for byte.

The printer reads the source and not the tree alone, because comments are not part of the tree.
Formatting preserves the tree all the same: `parse(format(source))` equals `parse(source)`.

The formatter has no options.
`docs/specs/formatting.md` states the canonical form it writes, construct by construct.
The snippets in this document illustrate the shape of each feature, not its canonical spelling.

### Naming

Canonical form covers how a name is spelled, not only where it is written.

**A function, a parameter, a record field, and an imported module are `snake_case`**.
**A type, a variant, and a type parameter are `PascalCase`**.
An acronym is a word, so `UserId` is canonical and `UserID` does not compile.

**A declared name is two characters or more**, because `f` names nothing a reader can look for.
A type parameter is exempt: it names no domain concept, and `T` is how that is written.

**A function whose result is `Bool` asks the question it answers**, beginning `is_`, `has_`,
`can_`, or `should_`.
`if is_active(user)` reads as a question where `active(user)` reads as a command.

The rules are about declared names, which is every name another file can write.
A local binding is private to the body it is written in, so none of them is about one.

Naming is checked and never rewritten, for the same reason order is: what a thing is called is the
author's decision, so the compiler says what canonical form spells it rather than spelling it.
`docs/specs/naming.md` is the specification.

### Order

Canonical form covers sequence, not only whitespace.

**Imports come first**, before every declaration, sorted by the module they name.

**A declaration is written above what it uses**, so a helper sits below the thing it helps.
A file reads top down: the reader meets the intent before the detail.
Two declarations that use each other are written either way, because no order undoes a cycle.

**A `match` lists its arms in the order the type declares its variants**.
A new variant then has exactly one place to be handled, and no diff is ever reorder-only.

Order is checked and never rewritten.
`lumen fmt` repairs whitespace, which is nobody's decision.
Where a declaration belongs is the author's, so the compiler says where rather than moving it.

---

## 14. Effects

A future language feature should make effects explicit.

Pure code:

```text
fn calculate_premium(policy: Policy) -> Premium {
    ...
}
```

IO:

```text
fn load_user(id: UserId) -> User ! IO {
    ...
}
```

Database access:

```text
fn save_user(user: User) -> Result<(), DbError> ! IO {
    ...
}
```

The purpose is to let the compiler distinguish pure domain logic from operations involving:

* IO
* networking
* filesystem access
* clocks
* randomness
* external services
* mutable global state

This feature should come after the basic language and type system are working.

### Resources are the same feature

A scoped resource, a file or a connection, needs two guarantees.
It is released on every exit path, and it is never used after release.
Go's `defer` and Java's try-with-resources give only the first; a closed handle can still escape.
The second needs the type system; the classic answer is linear types, which are ownership's family.
The intended answer is instead an escape check.
A resource-typed value may be passed as an argument but not returned, stored in a field, or sent.
It therefore cannot outlive the block that opened it.
An effect is a capability passed the same way, so effects and resources are one mechanism, not two.
Elsewhere most of the cost of such a check is closures, which capture capabilities silently.
Lumen has no anonymous functions, and a named function cannot capture a local.
The escape routes are therefore enumerable, and the rule stays one paragraph.
That is a standing reason to keep the no-closures rule when it feels inconvenient.
No syntax is committed.
The block that opens a resource compiles to try/finally, and `AutoCloseable` never surfaces.
It is a JVM interface, and the JVM is a target, not a model.

---

## 15. Concurrency

Lumen adopts Go's concurrency model whole: spawned functions, channels, and blocking calls.
There is no **`async`/`await`** and no function colouring.
A function that blocks is an ordinary function, called like any other.
There is one kind of function, and no caller ever has to ask which kind it holds.
The JVM's virtual threads make blocking cheap, so the language never needs a second kind.

Conceptually:

```text
spawn process_events()
```

Channels:

```text
fn produce(events: Channel<Event>) {
    events.send(load_events())
}

events := Channel.new()

spawn produce(events)

event := events.receive()
```

`spawn` takes a named function call, never an anonymous block.

The underlying implementation can use JVM threads and, where appropriate, virtual threads.

The language should hide most JVM concurrency boilerplate.

Eventually, channel types and concurrency primitives may gain additional static guarantees.


---

## 16. Modules and names

One file is one module, and the module is named by its file.
There is no module declaration: the file is the declaration, and every name it declares is public.
A module is one file and a file is small, so a private declaration waits for a case that needs one.

`import io` brings the module `io` into scope, and its names are reached through it:

```text
import io

fn greet(name: String) {
    io.print("Hello, " + name)
}
```

A reader who meets `io.print` knows where to look without knowing what else the file imports.
An unqualified import would take that away, so Lumen has none.

**One name has one definition**.
No two declarations of a module share a name, and no binding hides a name already in scope.
There is no overloading: within a module a name is one thing wherever it is written.
Searching for a name then finds its definition and its uses, with nothing else mixed in.

Types and values are named separately, which is what makes a newtype ordinary:

```text
type UserId = UserId(Int)
```

`UserId` names the type where a type is written and the constructor where a value is written.
The compiler never has to guess which of the two was meant.

A field of a record and a name of an imported module are each reached through something else.
Neither is a name in scope.
`user.name` is looked up in the record and `io.print` in the module, never in the file.

`docs/specs/modules.md` states the scopes, the prelude every module has, and the errors.

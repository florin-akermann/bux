# Lumen: Language Design

> Go's simplicity.
> Haskell's type system.
> Valhalla's values.
> A compiler written in Rust.

A small, statically typed language for building practical software.
It is built on Valhalla, the JVM's value classes, and every Lumen value is a value.
No type a program declares has identity, and equality is by state, only where a type asks for it.
No built-in type is special: a type a library declares can do everything `Int` can.
It inherits neither Java's object model, Rust's ownership model, nor Haskell's complexity.

The compiler is written in Rust and targets JVM bytecode, on JDK 28 or later.
The JVM is the first compilation target and nothing more; section 2 says what that rules out.

This document is the language specification; every change to the language is a change here first.
How the compiler is built and what ships when is in `docs/implementation.md`.
The questions every proposed feature must answer are in `docs/principles.md`.

---

## 1. Goals

The language combines six properties.

### Go

Take inspiration from Go's:

* small language surface
* readable syntax
* fast compilation
* straightforward tooling
* simple deployment model
* practical standard library
* preference for explicit, boring code
* one canonical formatting

### Erlang

Take one idea from Erlang, and take it whole:

* a process owns its state, and nothing else reaches that state
* a message is the only way one process reaches another
* a process is cheap, so a program writes as many as the work has parts

That idea is the whole of what Lumen takes.
Section 15 lists what Erlang does beside it, and Lumen refuses each one.

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
* syntactic sugar: `++`, `--`, `-=`, `*=`, `/=`, `%=`, a ternary `?:`
* anonymous functions
* async/await, or any other function colouring
* a crash, a supervisor, or a restart; section 15 says why Lumen needs none of them
* a link, a monitor, a process registry, or hot code loading
* a mailbox that grows without a bound, or a receive that searches one
* a channel, or any other message queue that belongs to no process
* identity, or an equality every type has whether or not it asked for one
* boxing a program can observe, or a built-in type that is special
* a special case: a type, an operator, or a function the rules exempt
* a panic, a trap, or any other operation without an answer for some of its input

### The JVM is a target, not a model

The JVM is where Lumen compiles first, and that is the whole of its authority over the language.
None of its constraints is inherited.
Not the object model: no identity, no `is_equal` on everything, no `hashCode`, no root class.
Not the eight primitive types that are special against every other; `Int` is a type like `User`.
Not boxing, which a program never observes, and not erasure, which is why a generic boxes.
What Lumen adopts instead is value semantics, in the shape Valhalla gives a value class.
That shape is no identity, no null, and equality by state, and every Lumen type already has it.
`docs/principles.md` asks of every feature whether the JVM leaks through it; this is the rule.

### Sugar is a second spelling, and a second spelling is a cost

A shorthand that writes what the language already writes buys nothing the type system can check.
A reader learns both spellings, canonical form has to choose between them, and every later feature
answers to two forms rather than one.
So `++`, `--`, `-=`, `*=`, `/=`, `%=`, and a ternary `?:` are not deferred; they are refused.
`a = a + 1` and an `if` say each of them, and say it in the one shape the rest of the language has.

`+=` is the one shorthand the language keeps, because a `for` loop that totals is the everyday
shape Lumen is built around, and section 8 makes it `Add` exactly as `+` is.
It is the ceiling rather than the first of a set: a second shorthand lands only where it removes a
class of mistake, never where it removes typing.
`docs/principles.md` question 9 is what any proposal for one answers.

### Nothing is a special case

A rule of this document holds for every type, every function, and every operator, or it is no rule.
A prelude type is a type a library could have declared, and `Int` has nothing a declared type lacks.
An operator is a function with other syntax, and section 5 gives it no exemption from its type.
`main` is a function like any other: it declares what it takes and gives back, and section 11
makes that `List<String>` and `Int`.
A feature that works only because one name is treated apart from the rest is reshaped or refused.
`docs/principles.md` question 10 asks it of every proposal.

### Nothing panics

An operation with no answer for some of its input says so in its type, and never at runtime.
Rust panics on `x / 0` and calls the panic a design; Lumen does not, because a crash is an untyped
answer.
There is no panic, no trap, no exception, and no `unwrap`: no runtime failure a program can reach.
Section 5 states the rule, and `docs/principles.md` question 11 asks it of every proposal.

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

A pattern is one of four things: a name that binds, `_`, a literal, or a constructor with
patterns inside it.
Alternatives are written in one arm as `Pending | Running`, which matches what either of them
matches and binds nothing.

A guard is refused.
An `if` inside the arm says the same thing, and a guarded arm is an arm no exhaustiveness check
can reason about.

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
There is no panic and no trap, so there is no runtime failure for a program to catch or to observe.
Rust panics on `x / 0` and calls the panic a design; Lumen refuses the trade, and the type answers.

A library module may sit on a JVM operation that throws, and gives back a `Result` where it does.
The throw is caught where the declaration that reaches the operation is written, and never reaches
the program.
`docs/specs/interop.md` states the catch, and `docs/specs/io.md` the one read version 0.1 has.

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

**`Option` never carries `()`.**
`Some(())` says only that a value is there, and `None` that it is not: `Bool` spelled a second way.
At the boundary to Java it is worse, because `Some` there also reads a `null` that did not come.
`Some(())` is then a flag for `null`, and nullability is a non-goal of section 2.
The compiler refuses `Option<()>` wherever a program writes it or inference reaches it.
`Result<(), E>` stays, because its `Err` carries a reason the caller could not work out.
`docs/specs/interop.md` states the boundary rule, and `docs/specs/types.md` the refusal.

There is no `unwrap` and no `expect`, in the prelude or anywhere else.
`maybe.or(fallback)` is the total default, named for what it does rather than for what it is not.

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
    fn is_equal(one: T, other: T) -> Bool
}
```

Generic functions can constrain their types:

```text
fn has_value<T: Eq<T>>(
    items: List<T>,
    value: T
) -> Bool {
    ...
}
```

A type gives a trait an instance:

```text
instance Eq<Point> {
    fn is_equal(one: Point, other: Point) -> Bool {
        one.across == other.across && one.down == other.down
    }
}
```

An instance writes a body for every method its trait declares, and for no other name.
Each body has the trait's signature with the trait's type parameter standing for the instance's
type, so an instance adds nothing the trait had not already said.
A trait declares one type parameter, which is what `Eq<T>`, `Ord<T>`, and `IntegerLiteral<T>` each
ask for and all the language has needed.
An instance is for a type written by name: `Eq<Point>`, and not `Eq<List<Point>>`.
The spec that derives an instance for a generic type settles how one is written for `List<T>`.

An instance belongs in the module that declares the trait or in the module that declares the type.
One trait and one type have one instance in a program, and a second is refused where it is written.
That is what lets a constraint reach an instance without anyone saying which one:
`T: Eq<T>` at `T = Point` reaches the one `Eq<Point>` there is.

A type parameter takes one constraint for each trait the body asks of it, joined by `+`.
`K: Eq<K> + Hash<K>` is what a hashed map asks of a key, and a use of it is accepted only where the
key's type has both instances.

A constraint resolves while the program is compiled, because a generic is compiled once per set of
types.
`has_value` at `Point` is a method whose body calls the `is_equal` of `Eq<Point>` and nothing else.
No dictionary is passed, no method table is built, and nothing about the call waits for the program
to run.
A trait method called at a type with no instance is refused where it is called, and a constrained
generic used at such a type is refused the same way.

`==` is `Eq`: a type is compared only when it has an instance, and a built-in type is no exception.
The library ships the three instances: `Int`, `Bool`, and `String`.
`==` on a record or a variant is refused until its type has one, which it writes or derives.

Every operator is a trait method, and `==` is only the first to be written that way.
`+` is `Add`, `-` is `Sub`, `*` is `Mul`, `/` is `Div`, `%` is `Rem`, and prefix `-` is `Neg`.
`<`, `<=`, `>`, and `>=` are `Ord`, and `&&`, `||`, and `!` stay `Bool`'s alone: they short-circuit.
An operator's type is its instance's type: `Div<Int>` gives `Option<Int>`, `Add<Int>` an `Int`.
The library ships the instances for `Int` and `String`; a declared type writes its own the same way.
A type without an instance has no operator, so `a + b` over two `UserId`s is refused, as it is now.
The wiring version 0.1 shipped, which named `Int` and `String` where an instance now stands, was
the degenerate case of this design rather than a design of its own, and it is gone.

A literal is a trait method too, so a declared type can be written as plainly as `Int` can.
A whole-number literal takes the type the context expects, provided that type has `IntegerLiteral`.
A literal that does not fit its type is a compile error where it is written.
It is never a wrapped value and never a runtime failure, because a literal is no exception.
A literal whose type nothing settles is an `Int`, which is the one default the language keeps.
`docs/specs/literals.md` settles how an instance states what fits: two bounds the compiler reads.

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

A map and a set are values too, and both are library types rather than language ones.

```text
ages := map.insert(map.empty(), "ada", 36)
found := map.get(ages, "ada")
```

`Map<K, V>` holds one value for each key it is given, and `Set<T>` holds a value once however
often it is given.
Neither has identity, as no record has: nothing can ask whether two of them are one object.
A key is a type a program compares and hashes, so `K` is constrained by `Eq<K>` and `Hash<K>`.
Each of the two is a hash array mapped trie: a branch holds thirty-two children, indexed by five
bits of the key's hash, and a leaf holds the entries whose keys hash alike.
`get` gives an `Option<V>`, because a key the map has no entry for is a case the type has to say.
There is no literal for either: a map is built by `empty` and `insert`, and read by `get`.
`docs/specs/collections.md` states each function, its type, and what it costs.

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

**Every type a program declares is a value with no identity**.
Nothing can ask whether two of them are one object, so nothing can be reached into from elsewhere.
Three things do have identity: a process, a scoped resource, and a foreign reference.
A `type` declaration writes none of them, and sections 14, 15, and 17 state what each one is.

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

active_names := users.filter(is_active).map(user_name)
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

### A call with its first argument in front

`maybe.or(fallback)` is the call `or(maybe, fallback)`, written with its first argument in front.
`or` is looked up in scope exactly as a plain call looks its callee up, and `maybe` is passed first.
The two spellings are one call: the same function, the same type, and every rule of this section.
Which function is called is settled by the name alone, before any type is known.

Nothing is declared to earn the form, so a declared type has it exactly as `Option` and `Int` do.
There is no method and no receiver type, because a function takes what it takes.
The dot moves nothing but the reader's eye.

The form reads in the order the work happens.
`find_user(id).or(guest)` says what is looked for before what stands in for it.
`or(find_user(id), guest)` says the same thing inside out.
A chain of calls then reads left to right, as the snippet above does.

The name before the dot says which of three things the dot does.
`user.name` reads a field, and `io.print("hi")` reaches a function of a module.
`maybe.or(0)` reaches `or` in scope, because `maybe` is a binding and a binding is never a module.
A field and a call are told apart by the parentheses: `user.or` reads a field named `or`.

The receiver is an argument, and it is never named.
`old.rename(to: new)` names one argument and not the other, so it is refused as the rule below says.
A call that has to name its arguments is written plainly.

The formatter keeps whichever form the author wrote.

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

The rule asks an author to have declared that two-variant type instead, so it holds where they
chose the types and nowhere else.
An instance method's signature is its trait's: `instance Hash<Bool>` writes `hashed(value: Bool)`
because `trait Hash<T>` wrote `hashed(value: T)` and the instance settled `T` on `Bool`.
There is no flag its author could have declined to write, and the two-variant type the rule asks
for is one the trait would have had to take.

So the rule reaches an instance method through its trait rather than at the instance.
A trait writing `fn hashed(value: Bool) -> Int` is refused where it writes it, because that `Bool`
is a flag every instance of the trait would then have to take.
Nothing is given up: the one place such a parameter can be written is the one place it is read.

### The entry point

A program starts at `main`:

```text
fn main(arguments: List<String>) -> Int {
    greet("world")
    0
}
```

`main` takes the words the program was run with and gives back the status the run ends with.
That is the one shape a program starts at: the one parameter is `List<String>`, and the result
is `Int`.
It is a function like any other, and the signature is written out, not a form it is spared.
A module declaring `main` at that shape can be run.
A module declaring `main` at any other shape is a library, exactly as one declaring no `main` is,
and running it is refused with a message naming the shape to write.

`arguments` holds every word written after the file, in the order the command wrote them.
The name of the program is not one of them, because a program already knows what it is.
The JVM starts at a `main(String[])` of its own; the compiler writes that, and no program sees it.
That entry point gathers the array into the `List<String>` it hands `main`.

A run is over when `main` is, and the `Int` it gave back is the status the run ends with.
A program that has nothing to say gives back `0`.
A status is eight bits wide on every system the JDK runs on, so the entry point hands the system
the low eight bits of that answer and nothing else.
Every `Int` maps to one status that way, so giving back a status is never a partial operation.
`docs/specs/run.md` states how a run passes the arguments and reads the status.

### Every function carries an example

**A function a module declares at the top level states at least one example**, or it is not built.

```text
// Divides `total` among `people`, giving back nothing where there is nobody to divide among.
//
// example: shared(total: 17, people: 5).or(0) == 3
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
A resource-typed value may be passed as an argument to an ordinary call, and nothing else.
It may not be returned, stored in a field, sent in a message, or given to a process.
It therefore cannot outlive the block that opened it.
A foreign reference is held to the same four clauses, and the reason is the object behind it.
That object has identity and mutates, so one a process holds is shared state again.
That is the escape check doing one job rather than a second rule written for the boundary.
An effect is a capability passed the same way, so effects and resources are one mechanism, not two.
Elsewhere most of the cost of such a check is closures, which capture capabilities silently.
Lumen has no anonymous functions, and a named function cannot capture a local.
The escape routes are therefore enumerable, and the rule stays one paragraph.
That is a standing reason to keep the no-closures rule when it feels inconvenient.
No syntax is committed.
The block that opens a resource compiles to try/finally, and `AutoCloseable` never surfaces.
It is a JVM interface, and the JVM is a target, not a model.

**A resource has identity, and a `type` declaration does not write one**.
The file it names is one file, and releasing it is a change every later use would see.
A process and a foreign reference are the other two, which sections 15 and 17 state.
Section 10 states the rule all three stand outside, which is about a type a program declares.
The escape check above holds two of the three, and a process handle is the one it lets be sent.

---

## 15. Concurrency

Lumen adopts Erlang's concurrency model.
A process owns its state, a message is the only way to that state, and a process is cheap.
There is no **`async`/`await`** and no function colouring.
A function that blocks is an ordinary function, called like any other.
There is one kind of function, and no caller ever has to ask which kind it holds.
A target must make blocking cheap, as the JVM's virtual threads do, so no second kind is needed.

Conceptually:

```text
type Request =
    | Add(Int)
    | Stop

process Counter {
    fn start(initial: Int) -> Int {
        initial
    }

    fn receive(total: Int, message: Request) -> Next<Int> {
        match message {
            Add(amount) => added(total, amount)
            Stop => Done
        }
    }
}

fn added(total: Int, amount: Int) -> Next<Int> {
    Continue(total + amount)
}

counting := spawn Counter(0)

_ = counting.send(Add(3))
```

`spawn` takes a `process`, never a function and never an anonymous block.
It gives back a handle, and that handle is the one name the rest of the program has for the process.
`Process<Request>` is that handle, and a program never names the queue behind it.
`spawn` runs `start` with the arguments the caller passes by value, and that result is the state.
A process accepts one type, which is why `Request` is an ADT and not a list of message kinds.
`match` on that ADT is how a process tells one message from another.

**Every process has one shape, and the compiler gives it no second one**.
Seen one process, seen them all: a reader knows where the state starts and where each message goes.
A `process` declares `start` and `receive`, in that order, and declares nothing else.
`start` builds the first state, and `receive` takes the state and one message and gives the next.
The prelude declares `Next<T>` as `Continue(T)` or `Done`, and a library could have declared it.
`Done` is how a process ends itself, and question 8 is answered because the type is an ordinary ADT.
The body of `receive` is one `match` on its message parameter, and nothing else.
Each arm of that `match` is one call or one name, and never a block, an `if`, or a second `match`.

That last rule is what keeps a process readable as it grows.
A branch cannot hold the work, so the author must lift the work out and give it a name.
The compiler holds the shape, and it does not judge the name it forced the author to write.
A rule about a good name is not a rule a compiler can hold, and Lumen does not pretend otherwise.

The shape is a declaration rather than a check over a loop a program writes.
Question 7 of `docs/principles.md` is why: elegance leaves the invalid case unwriteable.
A hand-written loop lets every author write a slightly different process.
A linter over that loop rejects each one after the fact, which question 7 calls adequacy.
There is no loop here to write differently, and no `var` to hold state beside the state.
There is no early `return` either, so no path leaves the process without a next state.

Question 12 holds a library method to a `for` loop, and a `process` declaration is no method.
Read broadly, the question still lands: a `for` loop could write this, and that is the problem.
A process is the one place where a second shape costs a reader the whole program.
A process is therefore written one way, and `spawn` takes no other.
A second form would be the second spelling that question 9 refuses.

**It is all messages**.
There is no mutex, no atomic, and no condition variable.
This is not a rule the compiler polices but a consequence of the value model.
A lock protects shared mutable state, and no type a program declares holds any.
Every type a program declares is a value with no identity, so two processes never hold one.
A record is rebuilt rather than reached into, as section 10 says.
A named function cannot capture a local, and `spawn` passes its arguments by value.
The one mutable thing, a `var` binding, is therefore never visible from another process.
There is no mutable global state, which section 14 lists as an effect for the same reason.
A foreign reference is no value a program declared either.
Section 14 refuses one given to a process, which is the clause that keeps it out.

**A queue belongs to a process, and Lumen has no channel**.
Go gives a queue an identity of its own, and any process may hold it.
Such a queue is shared mutable state with a name, which the value model refuses everywhere else.
It also needs rules that a mailbox does not need.
Nothing owns a channel, so closing one is a protocol rather than an event.
A reader holds several channels, so a `select` must choose among them.
A mailbox ends when its process ends, and one mailbox needs no `select` because `match` chooses.
One queue with many readers is the one shape a channel has and a mailbox does not.
A worker pool is that shape, and the answer is a process that takes jobs and hands them out.
That costs one process and one message, and a program writes it rather than the language.

**A process has identity, and a `type` declaration does not write one**.
The example above shows why: `counting` names one process, and a second `spawn` names another.
`send` puts a message in that process's queue, and the process reads it.
A scoped resource is the second and a foreign reference the third, which section 14 holds both of.
Section 10 states the rule all three stand outside, and a program has no type of its own to lock on.

**A slow receiver makes the sender wait**.
A mailbox holds a bounded number of messages, and `send` blocks while the mailbox is full.
That is the whole of backpressure, and every mailbox gives the same answer.
Erlang lets a mailbox grow without a bound, and one slow process then exhausts the memory.
Lumen refuses that, because the type system cannot see it and the program cannot recover from it.
A shared counter, cache, or pool is a process that owns the state and receives requests.
That is slower than a lock on a hot path, and the trade is accepted.
A lock-free structure is a platform library reached through `extern`, never something Bux writes.

**A message that cannot arrive is a typed result, never a silent drop**.
A process ends when `receive` gives `Done`, and its mailbox ends with it.
Erlang drops a message sent to a process that has ended, and it never tells the sender.
Lumen says so in the type of `send`, because section 5 gives every operation an answer.
The same rule refuses Erlang's crash: no Lumen process fails, so none needs a restart.
A supervisor, a link, and a monitor each answer a failure that Lumen's types answer first.
A process reports an end the sender must know about through a message, as every result travels.
Cancellation is a message, and the spec names the idiom rather than adding a primitive.

**A mailbox is read in order, and nothing searches it**.
Erlang's selective receive walks the mailbox for a message that matches and leaves the rest behind.
It reads well, and it costs one scan for each receive, which a full mailbox makes slow.
Lumen gives `receive` the next message, and a process that must wait holds that in its own state.
A `send` takes a deadline, and a deadline of none is the send that never waits.
That is what a bus that drops the oldest message needs, and no other primitive answers it.
A process never calls `receive` itself, so a deadline on the receiving side is not a value it holds.
How a process wakes when no message comes is the second open question below.

**An in-application message bus is a library type, not a primitive**.
A `send` names the process it reaches, and a program often has a message that whoever cares reads.
A bus is the process that closes that gap, and it is written in Lumen the day a program needs one.
It carries messages between the processes of one program, and it never leaves that program.
It holds the handles of its subscribers, and it gives every message to every one of them.
A subscriber joins by sending the bus one message.
A publisher sends to the bus and names no subscriber, so a publisher and a subscriber never meet.
A bus carries one message type, because a handle accepts one type and a mailbox reads one.
Whoever cares is whoever subscribed to that bus, and one ADT unions several kinds under one bus.
Subscribing is a message, and unsubscribing is the message beside it.
A subscriber that has ended makes the next `send` give the result above, and the bus drops it then.
Both paths are messages, so the bus needs no close protocol and no scoped resource of its own.

A bus chooses what a slow subscriber sees: the publisher waits, or a buffer drops the oldest.
The dropping bus reports the loss as a typed result on receive, never silently.
A latest-value cell is a third type, not a mode on the first; a policy knob is refused.
The derivation runs one way: a bus is processes and messages, and no process comes from a bus.
That is why the process is underneath and the bus on top.

A bus that crosses a network is a different thing, and the paragraph below says where it lives.

Distributed messaging is a platform library, never part of the language or its runtime.
Erlang makes a remote process look like a local one, and Lumen does not.
A network call fails where a local `send` does not, and one name for both hides that difference.
A transport that shares the mailbox's receive shape gets a library wrapper once a program needs one.

`process`, `spawn`, the handle, the send deadline, and cancellation are 0.4 work.
Each gets a spec under `docs/specs/` before it lands.
Two questions stay open for that spec.
The first is how a request carries the address to answer on.
A reply is a message, so the reply address is a handle, and a handle accepts one type.
The second is how a process wakes when no message comes, which a deadline on `receive` once gave.
The underlying implementation can use JVM threads and, where appropriate, virtual threads.
The language should hide most JVM concurrency boilerplate.

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

A module offers the types it declares as well as the functions, and each is reached the same way:

```text
import greeting

fn shout(said: greeting.Greeting) -> String {
    greeting.spelled(said) + "!"
}
```

`greeting.Greeting` is the type that module declares, written where a type is written.
A variant of it is reached the same way, so a `match` over one names `greeting.Pending`.
A type is a name like any other, and a module is what a name is reached through.

A field of a record and a name of an imported module are each reached through something else.
Neither is a name in scope.
`user.name` is looked up in the record and `io.print` in the module, never in the file.
`maybe.or(0)` is neither: its dot puts an argument in front, and `or` is looked up in scope.
Section 11 states that form.

An import names the file the module is written in, beside the file that writes the import.
`import greeting` therefore reads `greeting.lm` from the same directory, and nowhere else but
a package this one depends on.
A ring of imports is refused: a module is compiled after what it imports, and a ring has no such
order.

`docs/specs/modules.md` states the scopes, the prelude every module has, and the errors.

A package is a directory of modules, named by a manifest written beside them.

```text
package shapes
version 0.2.0
depends ../geometry
```

An import that reaches nothing beside the file that wrote it reaches a module of a package this
one depends on.
Nothing is fetched: a dependency is a directory that is already there, and the manifest names it.
Two dependencies holding a module of one name are refused, because one name has one definition.
`docs/specs/packages.md` states the manifest, the order an import is answered in, and the errors.

---

## 17. Reaching the platform

A platform has libraries that are worth reaching, and the model that comes with them is not.
A boundary lets a program use those libraries, and it keeps their model out of the language.
An `extern` declaration is the one place a member of the platform is named.
It names exactly one member, and it gives that member a Bux signature.
From there it is a function like any other.

Each target has its own `extern` form, and a spec states that form.
The form says which kinds of member a declaration names, and how the declaration names each kind.
The compiler reads nothing of the platform, so the form also says which facts a declaration writes.
`docs/specs/interop.md` states the form of the JVM, which is the first target.
The rest of this section holds for every target, so a second target adds a spec and nothing here.
This example is the JVM form, as that spec states it:

```text
extern static read_string(path: Path) -> Result<String, String> = "java.nio.file.Files.readString"
```

Every parameter and every result is a Bux type.
There is no subtyping and no implicit conversion.
So a member that takes a wider type than the caller holds is not reachable.
The answer is to name a member that takes what the caller holds.
That is what keeps the boundary a signature rather than a second type system.

A fact about a member that no Bux type states is written in the declaration, if the form has it.
The width of a number that a member gives back is one example.
Such a fact is no Bux type, no program can write a value of it, and section 2 stays in force.
It is written where every other fact about the member is written.

Three things cross, and nothing else does.
A value of a Bux type crosses as itself.
An absent value that the platform gives back becomes `None`, where the result is an `Option`.
A failure that the platform reports becomes `Err`, where the result is a `Result`.
That `Err` holds what the platform says of the failure.

An extern type has no identity a program can reach, as every type a program declares has none.
A foreign value is held, handed on, and given back.
It is compared, hashed, or shown only where a trait instance over `extern` declarations says how.
There is no `equals`, no `hashCode`, and no `toString` reaching it, and no hierarchy above it.
Section 2 is not suspended inside the boundary.

What a program cannot reach, the foreign value still has: it has identity, and it mutates.
That is why section 14 names a foreign reference among what its escape check refuses.
One a process holds is shared mutable state, which section 15 has no answer for.
The clause already written is that answer, rather than a rule of the boundary's own.
An `extern` declaration is where a foreign reference comes from, and the check is what it goes to.

An `extern` declares what it can fail with, and that claim is the author's, not the compiler's.
It is the one claim in the language that nothing checks.
That is why the boundary is narrow and lives in the library.
`io` and `files` are Bux modules over `extern` declarations.
A program reaches the platform through them, rather than through an `extern` of its own.

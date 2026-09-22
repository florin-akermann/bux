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
* syntactic sugar: `++`, `--`, `-=`, `*=`, `/=`, `%=`, a ternary `?:`
* anonymous functions
* async/await, or any other function colouring
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
`main` is a function like any other: it declares what it gives back, and section 11 makes that `()`.
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
A key is a type equality is written over, so `K` is constrained by `Eq<K>` and by nothing else.
Hashing asks for a table to bucket into, which asks for an array the language cannot yet name,
and a constraint no body reads is a promise a caller keeps for nothing.
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
Three things do have identity: a channel, a scoped resource, and a foreign reference.
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
fn main() -> () {
    greet("world")
}
```

`main` takes nothing and gives back nothing: a program is run for what it does.
It is a function like any other, and `-> ()` is its return type written, not a form it is spared.
The JVM starts at a `main(String[])` of its own; the compiler writes that, and no program sees it.
A module declaring it can be run; one that does not is a library, and running it is refused.

A run is over when `main` is.
There is no exit status to write, because a program has nothing to say yet about how it went.

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
It may not be returned, stored in a field, sent on a channel, or passed to a spawned function.
It therefore cannot outlive the block that opened it.
A foreign reference is held to the same four clauses, and the reason is the object behind it.
A Java object has identity and mutates, so one a spawned function holds is shared state again.
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
A channel and a foreign reference are the other two, which sections 15 and 17 state.
Section 10 states the rule all three stand outside, which is about a type a program declares.
The escape check above holds two of the three, and a channel is the one it lets across a spawn.

---

## 15. Concurrency

Lumen adopts Go's spawned functions, channels, and blocking calls, and refuses Go's locks.
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

**A channel is the only way two spawned functions communicate**.
There is no mutex, no atomic, and no condition variable.
This is not a rule the compiler polices but a consequence of the value model.
A lock protects shared mutable state, and no type a program declares holds any.
Every type a program declares is a value with no identity, so two functions never hold one.
A record is rebuilt rather than reached into, as section 10 says.
A named function cannot capture a local, and `spawn` passes its arguments by value.
The one mutable thing, a `var` binding, is therefore never visible from another spawned function.
There is no mutable global state, which section 14 lists as an effect for the same reason.
A sender never knows its receivers, and no value a program declared is ever shared.
A foreign reference is no value a program declared either.
Section 14 refuses one passed to a spawned function, which is the clause that keeps it out.

**Three things have identity, and a `type` declaration writes none of them**.
The example above shows the first: `produce` sends on the `events` the parent holds.
`send` changes the queue behind the channel, and `receive` sees the change.
Two holders of one channel is the whole of what a channel is for, so a channel has identity.
A scoped resource is the second and a foreign reference the third, which section 14 holds both of.
Section 10 states the rule all three stand outside, and a program has no type of its own to lock on.

**A one-place channel is a lock, and the language gives no other**.
Go writes a mutex that way, and Lumen has no reason to refuse it.
It is a channel like any other: it blocks, it is not reentrant, and there is nothing else to learn.
What the language refuses is a second mechanism beside the channel, not this use of the channel.

**A slow receiver makes the sender wait**.
That is the whole of backpressure, and every channel gives the same answer.
A shared counter, cache, or pool is a spawned function that owns the state and receives requests.
That is slower than a lock on a hot path, and the trade is accepted.
A lock-free structure is a JVM library reached through interop, never something Lumen writes.

Channels alone are not enough; three things arrive with them, or programs reinvent locks badly.
A `select` chooses over several channels; a timeout, a cancellation, and a fan-in are all `select`.
A channel closes, and `for … in` over it ends when it does, so a producer can say it is finished.
Cancellation is a channel that closes; the spec names the idiom rather than adding a primitive.
These are 0.3 work, and each gets a spec under `docs/specs/` before it lands.

**Fan-out is a library type, not a primitive**.
A topic delivers every message to every subscriber.
It is a spawned owner holding subscriber channels, written in Lumen when a program needs it.
A topic chooses what a slow subscriber sees: the sender waits, or a bounded buffer drops the oldest.
The dropping topic reports the loss as a typed result on receive, never silently.
A latest-value cell is a third type, not a mode on the first; a policy knob is refused.
A subscription is a scoped resource, released as section 14 releases a file, and yields a channel.
Releasing it is what unsubscribes, and a subscriber has no name to hand back instead.
The derivation runs one way: a worker pool takes each job exactly once, which fan-out cannot say.
That is why the channel is underneath and the topic on top.

Distributed messaging is a JVM library a program consumes, never part of the language or runtime.
A transport that shares the channel's receive shape gets a library wrapper once a program needs one.

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

## 17. Reaching Java

The JVM ecosystem is worth reaching, and the object model that comes with it is not.
An `extern` declaration is the one place a Java member is named, and it names exactly one.
It gives that member a Lumen signature, and from there it is a function like any other.

```text
extern type Path = "java.nio.file.Path"

extern static read_string(path: Path) -> Result<String, String> = "java.nio.file.Files.readString"
```

There is one form per kind of member the JVM has, and no sixth kind to learn.
`type` names a class a value is held as, `field` a static field, `static` a static method,
`method` an instance method whose receiver is the first parameter, and `new` a constructor.
A declaration says which it is, so nothing about a call is inferred from the shape of its
signature.

Every parameter and every result is a Lumen type, and the JVM class it compiles to is the one the
member's own descriptor names.
There is no subtyping and no implicit conversion: a member taking `java.lang.Object` is not
reachable, and the answer is to name one that takes what the caller holds.
That is what keeps the boundary a signature rather than a second type system.

One width is the exception, and the declaration is what states it.
`Int` compiles to a `long`, and a great many Java members give back an `int` instead, `hashCode`
and `length` among them.
Which of the two a member gives is written in that member's own class file, and the compiler
reads none, so the author says it by writing `int` after the kind:

```text
extern method int length(text: String) -> Int = "length"
```

The member is called for its `int` and the answer is widened to the `Int` the signature declares.
Nothing else changes: `int` is no Lumen type, no program can write one, and section 2 is not
suspended to let one out.
It is a fact about the member, written where every other fact about the member is written, and
the only thing refused is writing it where the result is not an `Int` to widen to.

Three things cross, and nothing else does.
A value of a Lumen type crosses as itself.
A `null` given back becomes `None`, which is what an `Option` result declares.
Anything thrown becomes `Err`, holding what the throwable says of itself, which is what a `Result`
result declares.

An extern type has no identity a program can reach, as every type a program declares has none.
A Java object is held, handed on, and given back, and it is compared, hashed, or shown only where
a trait instance written over `extern` declarations says how.
There is no `equals`, no `hashCode`, and no `toString` reaching it, and no class hierarchy above
it: section 2 is not suspended inside the boundary.

What a program cannot reach, the object still has: a Java object has identity, and it mutates.
That is why section 14 names a foreign reference among what its escape check refuses.
One a spawned function holds is shared mutable state, which section 15 has no answer for.
The clause already written is that answer, rather than a rule of the boundary's own.
An `extern` declaration is where a foreign reference comes from, and the check is what it goes to.

An `extern` declares what it can fail with, and that claim is the author's rather than the
compiler's.
It is the one claim in the language nothing checks, which is why the boundary is narrow and lives
in the library: `io` and `files` are Lumen modules over `extern` declarations, and a program
reaches Java through them rather than through `extern` of its own.
`docs/specs/interop.md` states the declaration, the two mappings, and the errors.

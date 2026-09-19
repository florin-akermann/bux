# Lumen: Language Design

> Go's simplicity.
> Haskell's type system.
> The JVM's runtime.
> A compiler written in Rust.

A small, statically typed language for building practical software on the JVM.
It inherits neither Java's object model, Rust's ownership model, nor Haskell's complexity.

The compiler is written in Rust and initially targets JVM bytecode.

This document is the language specification; every change to the language is a change here first.
How the compiler is built and what ships when is in `docs/implementation.md`.
The questions every proposed feature must answer are in `docs/principles.md`.

---

## 1. Goals

The language combines four properties.

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
* checked exceptions
* macros
* complicated metaprogramming
* implicit runtime magic
* excessive syntax
* anonymous functions
* async/await, or any other function colouring

The guiding principle is:

> Strong static guarantees, without the programmer understanding the runtime's memory model.

---

## 3. Core design

A program should look approximately like this:

```text
type UserId = UserId(Int64)

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
Int64
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

The `?` operator propagates an error.

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

A name forces the author to say what the function is for, and gives the reader a word to search.

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

---

## 15. Concurrency

Lumen adopts Go's concurrency model whole: spawned functions, channels, and blocking calls.
**There is no `async`/`await` and no function colouring.**
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
type UserId = UserId(Int64)
```

`UserId` names the type where a type is written and the constructor where a value is written.
The compiler never has to guess which of the two was meant.

A field of a record and a name of an imported module are each reached through something else.
Neither is a name in scope.
`user.name` is looked up in the record and `io.print` in the module, never in the file.

`docs/specs/modules.md` states the scopes, the prelude every module has, and the errors.

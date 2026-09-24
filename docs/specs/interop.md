# Reaching a Java class

An `extern` declaration names one member of one Java class and gives it a Bux signature.

## Intent

`docs/design.md` section 17 states the boundary for every target.
It says that each target has its own `extern` form, and this spec is the form of the JVM target.
The member kinds, the widths, `interface`, and the two mappings are JVM facts, so they are here.
`docs/implementation.md` section 3 asks for the boundary.
`io` and `files` were the compiler's own modules because each reaches a JVM method no Bux source
could name, and they are library modules written over these declarations once one can.
This spec says what an `extern` declaration is, what crosses the boundary, and what is refused.

The boundary is a signature and nothing more.
Java's object model does not come with it: there is no subtyping, no overload to pick between,
no class hierarchy, and no `equals`, `hashCode`, or `toString` reaching a value.
Two widths cross, `int` and `char`, each widened to the `Int` a signature declares, and the
declaration is what says so.
What a declaration cannot say is not reachable, and the answer to that is another declaration.

## What a declaration is

There is one form per kind of member the JVM has, and no sixth kind.

```text
extern type File = "java.io.File"
extern type interface Path = "java.nio.file.Path"
extern type PrintStream = "java.io.PrintStream"

extern field out() -> PrintStream = "java.lang.System.out"
extern static read_string(path: Path) -> Result<String, String> = "java.nio.file.Files.readString"
extern method to_path(file: File) -> Path = "toPath"
extern method int length(text: String) -> Int = "length"
extern method char at(text: String, int index: Int) -> Option<Int> = "charAt"
extern new opened(name: String) -> File
```

`type` names a class, under a Bux type name that values of it are held as.
`field` names a static field, and reading it is a call of no arguments.
`static` names a static method.
`method` names an instance method of a class, whose receiver is the first parameter.
`new` names a constructor, of the class its result is.
`interface` after `type` says the class is an interface rather than an ordinary class.
A width after the kind says what the member's own descriptor gives back, and a width before a
parameter's name says what it takes; the two sections on the widths state both.

A declaration says which kind it is, so nothing about a call is worked out from the shape of a
signature.
`field` and `static` name the class as well as the member, because neither has a receiver to name
it.
`method` names the member alone, because the receiver's type already says which class it is on,
and `new` names neither, because its result already says.
Each of those is a disagreement that cannot be written rather than one that is reported.

Everything else about an `extern` function is an ordinary function of the module that declares it.
It is reached through that module's name, it is part of the module's surface, and a call of it is
a call like any other.

## Which classes a declaration may name

`docs/design.md` section 17 says that a declaration names only what the build can account for.
On the JVM, that is a class in an archive the package states, or a class on the list below.
A `type`, a `field`, and a `static` each name a class, and each such class is held to this rule.
A `method` reaches the class of its receiver, and a `new` the class it gives back.
Each of those is `String` or an extern type, and its own declaration is held to the rule already.
A class that is neither in a stated archive nor on the list is `L0434`.

### A class in a stated archive

The archives are the ones a `jar` line states, which `docs/specs/packages.md` states.
They are the archives of the package that is built and of each package it depends on.
That is the class path a run reaches, which `docs/specs/run.md` states, and nothing more.
The build holds each archive to its line, then opens it once and asks it for each class.
The class `com.example.Clock` is there if the archive has the entry `com/example/Clock.class`.
A nested class `a.B$C` is the entry `a/B$C.class`, as the JVM writes it.
A build whose modules name no class outside the list opens no archive for this check.

An archive accounts for no class in a package that the JDK holds.
Such a package starts with `java.`, `javax.crypto.`, `javax.net.`, `javax.security.`, or `jdk.`.
It can also start with `sun.` or `com.sun.`.
The JVM loads such a class from the JDK and not from the archive, so the archive cannot supply it.
So an archive with the entry `java/lang/reflect/Method.class` does not account for that class.

### The list of `java.base`

A class of `java.base` is on the list when its package is on the list and the class is not out.
The packages on the list are these:

- `java.io`
- `java.lang`
- `java.nio`
- `java.nio.charset`
- `java.nio.file`
- `java.util`
- `java.util.concurrent`
- `java.util.jar`
- `java.util.stream`
- `java.util.zip`

A package is on the list only by its whole name.
So `java.lang.invoke` and `java.lang.reflect` are not on it, and `java.net` is not on it.
A class of a module other than `java.base`, such as `javax.naming.InitialContext`, is on no list.

These classes of a listed package are out, each with every class nested in it:

- `java.io.Externalizable`, `java.io.ObjectInput`, `java.io.ObjectInputFilter`
- `java.io.ObjectInputStream`, `java.io.ObjectOutput`, `java.io.ObjectOutputStream`
- `java.io.ObjectStreamClass`, `java.io.Serializable`
- `java.lang.Class`, `java.lang.ClassLoader`, `java.lang.ClassValue`
- `java.lang.Module`, `java.lang.ModuleLayer`, `java.lang.StackWalker`
- `java.util.ResourceBundle`, `java.util.ServiceLoader`

Each of them does one of three things that the list keeps out of a program.
It loads a class by a name that the program holds only at run time.
Or it reaches the members of a class by reflection.
Or it makes an object out of serialized bytes.

These members of a listed class are out, and every other member of their class is on the list:

- `java.lang.Runtime.exec`, `java.lang.Runtime.halt`
- `java.lang.Runtime.load`, `java.lang.Runtime.loadLibrary`
- `java.lang.System.load`, `java.lang.System.loadLibrary`

A `field` and a `static` name their member with its class.
A `method` names its member, and the class of its receiver is known from the extern type.
So each of the three kinds is held to this second list, and a member that is out is `L0434` too.

The list is a table in `compiler/boundary.bx`, and the lists above are that table.
A program cannot add to it, and a manifest cannot add to it.
A class that a program needs and that is not on the list comes from an archive the package states.
The compiler and the runner are Bux programs, so they obey the same list.
The compiler reads its resources without a class loader, which `docs/specs/library.md` states.

## What crosses

A parameter and a result are Bux types, and each compiles to exactly the JVM type the member's
own descriptor names.

| Bux         | JVM                     | where          |
|---------------|-------------------------|----------------|
| `Bool`        | `boolean`               | either         |
| `Int`         | `long`                  | either         |
| `String`      | `java.lang.String`      | either         |
| an extern type | the class it names      | either         |
| `()`          | `void`                  | a result       |
| `Option<T>`   | what `T` is, or `null`  | a result       |
| `Result<T, String>` | what `T` is, or thrown | a result |
| `List<T>`     | `java.util.List`        | a parameter    |

Nothing else crosses.
A record, a variant, and a type parameter are each a Bux type a Java member has no descriptor
for, and an `extern` naming one is `L0425`.
The JVM's `double` and the rest of its primitives are types no Bux type compiles to, so a
member whose descriptor names one is not reachable and the answer is to name one that does not.
The two widths are the exception, and the sections on them state each one.

There is no subtyping and no implicit conversion.
So a member that takes `java.lang.Object` is not reachable from a program that holds a `String`.
The answer is to name a member that takes what the caller holds.
That is what keeps the boundary a signature rather than a second type system.

A `List<T>` crosses as a `java.util.List` that holds what the list holds, and nothing more.
`docs/specs/codegen.md` states the copy that builds it, so the member cannot change the Bux list.
It is a parameter and never a result.
A list a member gives back is a JVM object that member may still reach through and change, and
`docs/design.md` section 2 keeps a value that something else can change out of the language.
What a list holds is held to this same table, so `List<String>` and `List<Int>` each cross and
`List<User>` does not.
The JVM writes no element type into a descriptor, so which Java type the elements are is the
author's claim, exactly as the member itself is.

An extern type is a type like any other from where a program stands, except where it may go.
It is matched by nothing, because it declares no variants and no fields.
It has no identity a program can reach, as every type a program declares has none.
It is compared, hashed, or shown only where an `instance` over `extern` declarations says how.
A `derive` naming one is `L0427`: a derive reads what a type holds, and this one holds the JVM's.

## Where a foreign reference goes

What a program cannot reach, the Java object still has: identity, and mutation.
So a value of an extern type is a foreign reference, and it stays in the function that reaches it.
Section 14 of `docs/design.md` states that escape check, and this section states what is built.

A foreign reference goes to these places, and to no other:

- An `extern` declaration reaches it, and a call of that declaration gives it back.
- A `let`, a `var`, a parameter, a `match` binding, or a `for` binding holds it.
- A call takes it as an argument.

A type is or holds an extern type where it is one, or where a type argument is or holds one.
It holds one too where a field or a variant position its declaration writes is or holds one.
The check follows each declared type once, exactly as `L0811` follows it.

`L0435` refuses a foreign reference in two kinds of place, and its message names the place.

The first kind is a declaration:

- the result of a function that the program declares, as inference settles it;
- a field of a record, or a field of a variant;
- a position of a variant.

The span is the written type, or the name of a function whose result is not written.
The result of an `extern` declaration is not refused, because that is how a program reaches one.
So `Option<T>` and `Result<T, String>` over an extern type stay results of an `extern`.
The two mappings below state them.

The second kind is a call of a function or a constructor whose settled result is or holds one.
That closes the route through a type parameter.
For `fn same<T>(value: T) -> T`, `same(file)` is refused.
`Some(file)` is refused, and so is `Box { value: file }` for `type Box<T> = { value: T }`.
A call of an `extern` declaration is not refused, and the span of a refused call is the whole call.
The check knows a call of an `extern` by the result that its callee declares.
Only an `extern` may declare a result that holds one, because the first kind refuses the rest.

A name, `?`, `match`, `if`, and a list literal are not calls, and none of them is refused.
Each of them moves the reference inside the function, and nothing more.
Every route out of the function is a result, a field, a call, or a process.
The first three are `L0435`.
The fourth is `L0811`: a process holds no foreign reference, as `concurrency.md` states.

## The two widths

`Int` compiles to a `long`, and a Java member that gives back a number often gives an `int`
instead: `String.length`, `String.hashCode`, and `List.indexOf` each do.
Which of the two a member gives is written in that member's own class file, and the compiler reads
none, so the declaration says it by writing `int` after the kind:

```text
extern method int length(text: String) -> Int = "length"
extern method int hashed_text(value: String) -> Int = "hashCode"
```

`library/prelude.bx` writes the second of those, and `instance Hash<String>` is written over it.

The member is reached for its `int` and the answer is widened to the `Int` the signature declares.
`int` is no Bux type, no program can write one, and no type a signature writes compiles to one.
It is a fact about the member, written where every other fact about the member is written.

`char` is the other width, and a result reads it exactly as it reads `int`.
`String.charAt` gives back a `char`, which is no type Bux has and no type a signature writes.
`char` after the kind says the member's own descriptor gives one, whose descriptor letter is `C`,
and the answer is widened to the `Int` the signature declares:

```text
extern method char at(text: String, int index: Int) -> Option<Int> = "charAt"
```

A declaration writes at most one width word on its result, because a member gives back one value.

One thing is refused: a declaration writing a width whose result is not `Int`, because there is
then nothing to widen to.
That one rule holds for both words and for every kind, and a `new` writing a width meets it the
way any other does, since the class a constructor gives back is not `Int`.
`L0429` is what says so.

`Result` composes with a width as it does with anything else.
A result declared `Result<Int, String>` and written `int` guards the member, widens what it gave
back, and wraps that.
`Option<Int>` is no result of a declaration that narrows no parameter, with a width or without
one, because a number is never `null`; the next section states the declaration it is a result of.

## The width a parameter is narrowed to

`String.charAt` takes an `int` as well as giving back a `char`, and `String.substring` takes two.
A parameter says so by writing `int` before its name, as a result writes a width after the kind:

```text
extern method char at(text: String, int index: Int) -> Option<Int> = "charAt"
extern method cut_out(text: String, int from: Int, int to: Int) -> Option<String> = "substring"
```

The parameter's Bux type stays `Int`, and a caller hands over the whole number it always did.
The member is reached for an `int` there, so the argument is narrowed to one.

Narrowing a `long` to an `int` loses whatever does not fit, and nothing in Bux is partial, so the
result of a declaration that narrows a parameter is an `Option`.
Before the member is reached, each narrowed argument is compared with the `int` range.
If any argument is outside that range, the answer is `None` and the member is not reached at all.
Otherwise each narrowed argument becomes an `int`, the member is called, and the answer is `Some`
of what it gave back.

`L0431` refuses a declaration that narrows a parameter and gives back anything but an `Option`.
`Option<Int>` is a result of such a declaration, and so is `Option<T>` for every other `T` that
crosses as a value, because there is now something for `None` to say.
A declaration that narrows no parameter keeps the rule above: `Option` over a number is `L0425`.
`Option<()>` is `L0432` wherever it is written, which `docs/specs/types.md` states.
So a member that gives nothing back and takes an `int` is not reachable in version 0.1.

`Result<Option<T>, String>` composes as it always did.
The guard is outside, so an argument that does not fit is `Ok(None)`.
Where the result is a reference, a `None` still also reads a `null` the member gave back, and the
two reasons for a `None` compose into the one answer.

A width is a word and not a keyword, wherever it is written.
It is read as the width only where a name follows it, so `extern static int(text: String) -> Int`
declares a function named `int` as it always did, and a parameter named `int` is still `int: Int`.
`int` and `char` are ordinary names wherever a program writes one.

## Which kind of class it is

The JVM calls an instance method of a class one way and a method of an interface another.
Which of the two a Java name is is written in that name's own class file, and the compiler reads
none, so the declaration says it by writing `interface` after `type`:

```text
extern type interface Path = "java.nio.file.Path"

extern method as_text(path: Path) -> String = "toString"
```

A `method` whose receiver is such a type is called the way the JVM calls an interface's method.
That is the whole of what the word changes, and it changes nothing a program can see: the call
is written the same, the signature is the same, and the value is held the same.

A `field` reaches an interface as it reaches a class, because the JVM names a field one way for
both, and an interface's field is a static one like any other a `field` reads.

One thing is refused: a `new` whose result is such a type.
A constructor is the one member an interface has none of, so the declaration reaches nothing to
call, and `L0430` is what says so.

One thing waits: a `static` of an interface.
The JVM names a static method of an interface apart from a static method of a class, the way it
names an instance method of each apart.
An `extern static` names its class in the string rather than through an `extern type`, so there
is nothing to write the word on, and a declaration naming one is a class file that will not link.
That is the author's claim failing, and it waits for a requirement that has such a member to
reach.

The word is a word and not a keyword.
It is read as the kind only where a name follows it, which is the rule the width is read by, and
`interface` is an ordinary name wherever a program writes one.
A type called `interface` is one `docs/specs/naming.md` refuses on its own, because it spells a
type in `PascalCase`, so the word takes no name a program could have used.

## The two mappings

`Option<T>` and `Result<T, String>` are what the two things Java gives back that Bux has no
word for become.

A result declared `Option<T>` reads the reference the member gives back.
A `null` is `None`, and anything else is `Some` of it.
`T` is what the descriptor names, so `Option<Int>` is not a result: a `long` is never `null`.

A result declared `Result<T, String>` guards the call.
What the member gives back is `Ok` of it, and anything thrown is `Err` holding what the throwable
says of itself.
That is the one place the compiler catches anything, and it catches to build the `Result` and for
nothing else.

Neither wraps `()`.
`Ok` and `Some` each carry one value, and a member that gives nothing back leaves none to carry.
So `Result<(), String>` is `L0425`, and the answer is a member that gives something back.
`Option<()>` is `L0432`, because no program writes it, and that refusal comes before this check.
A `field` is the one kind `()` is no result for at all: the JVM has no field of type `void`, so a
field holds a value or is no field, and `L0425` says it holds none rather than gives none back.
Guarding a member that gives nothing back waits for a requirement that has one to guard.

The two compose: a result declared `Result<Option<T>, String>` is guarded, and the reference
inside the `Ok` is read for `null`.

A member that can throw and is declared without a `Result` throws through the program.
That claim is the author's and not the compiler's, and it is the one claim in the language nothing
checks.
It is why the boundary is narrow and lives in the library, and why `docs/specs/library.md` has a
program reach Java through `io` and `files` rather than through an `extern` of its own.

## What the bytes look like

An `extern` declaration is a static method of the module's class, written with the signature the
declaration gives it.
A call of one is the same static call as a call of any other function of that module, which
`docs/specs/codegen.md` states, so nothing about reaching Java is special at a call.

The body is the member and the mapping, and nothing else.
`field` reads the static field, `static` calls the static method, `method` loads the receiver and
calls the instance method, and `new` builds the class and calls its constructor.
A guarded body is a method of its own by construction, because every `extern` is one: a guarded
span begins with an empty stack, and a call may be written wherever an expression is.

Nothing of this is visible from Bux.
A program that writes `io.println` never learns what carries it, exactly as before.

A `method` is called the way an instance method of a class is called, and a `method` whose
receiver is an `extern type` written `interface` is called the other way, with the constant pool
entry an interface method reference is.
The declaration is the only thing that says which of the two a class file names, and one that
says the wrong one is the author's claim failing the way naming a member the JVM does not have is.

## The errors

| name                  | code    | message                                                    |
|-----------------------|---------|------------------------------------------------------------|
| type does not cross   | `L0425` | `User` is no type a Java member takes                      |
| not a Java name       | `L0426` | `java..File` is no Java name                               |
| derive of an extern type | `L0427` | `File` is an extern type, and a derive reads what a type holds |
| no class to reach     | `L0428` | a `method` reaches a class, and this signature names none  |
| no `int` to widen     | `L0429` | `int` widens to an `Int`, and this signature gives back `File` |
| builds an interface   | `L0430` | a `new` builds a class, and `Path` is an interface          |
| narrows, no `Option`  | `L0431` | an `int` parameter narrows an `Int`, and this gives back `Int` |
| no account of a class | `L0434` | `java.lang.Class` is a class this build cannot account for |
| a reference escapes   | `L0435` | `opened` gives back `File`, and the `extern` type `File` is a foreign reference |

`L0425` helps with ``a boundary carries `Bool`, `Int`, `String`, a `List`, and a type an `extern`
names``.
It points at the type as the signature writes it, and is raised for a result as well as for a
parameter, with `()`, `Option`, and `Result` accepted only as a result.
The message says what the member does with a value written there — takes it, gives it back, or,
for a `field`, holds it — because one signature can write all three.

`L0426` helps with `a Java name is its segments, each a name, with a dot between two of them`.
A name is refused where a segment is empty, where it begins with a digit, or where it holds
anything but letters, digits, `_`, and `$`.
A letter is a character that Unicode calls alphabetic, and a digit is one it calls numeric.
A `field` and a `static` name at least two segments, because the last is the member and the rest
is the class; a `type` names at least one; a `method` names exactly one.

`L0427` helps with ``write an `instance` over `extern` declarations instead``.

`L0428` helps with ``a class is `String`, or a type an `extern type` declares``.
A `method` reads its class off its first parameter and a `new` off its result, so each of those
has to name one; a signature that names none leaves the declaration with no class to reach, and
that is refused where it is written rather than met as a state the lowering has no answer for.

`L0429` helps with ``a width widens to an `Int`; drop the word, or give an `Int` back``.
It is raised where a `new` writes a width and where the result a member gives back, with a
`Result` or an `Option` around it read through, is not `Int`.
It reads `char` exactly as it reads `int`, because each of the two widens to the same `Int`.
A parameter meets the rule from the other side: one written `int` whose type is not `Int` has no
`Int` to narrow, and there the help reads ``an `int` narrows an `Int`; drop the word, or take an
`Int```.

`L0430` helps with ``a `new` builds a class; reach one through a member of a class instead``.
It is raised where a `new` gives back a type an `extern type` wrote `interface`, with a `Result`
or an `Option` around it read through, because a constructor is the one member an interface has
none of.

`L0431` helps with ``give back an `Option`, which is `None` where an argument does not fit``.
It is raised where a declaration narrows a parameter and the result, with a `Result` around it
read through, is not an `Option`, because a narrowed argument that does not fit has nothing to
say otherwise.
A result of `()` is refused by it too, and has no answer: the help asks for an `Option`, and
`Option<()>` is then `L0432`.

`L0434` helps with ``a class is in an archive a `jar` line states, or on the list of `java.base`
in `docs/specs/interop.md`, and this one is in neither``.
It points at the Java name, and its message names the class.
A member that the list leaves out is `L0434` with another message and help.
The message is ``the member `java.lang.System.load` is one the list of `java.base` leaves out``.
The help is ``the list in `docs/specs/interop.md` leaves this member out; reach another member``.

The help of `L0435` has two clauses with a `;` between them.
The first is ``a foreign reference stays in the function that reaches it``.
The second is ``bind it, and pass it as an argument``.
Its message begins with the place, then names the type there and the extern type it holds.
The place is `` `opened` gives back``, `` `Kept.file` holds``, or `` `There.0` holds``.
For a call it is ``this call of `same` gives back``, and for a record literal ``this `Box` is``.
A field of a record is named with its type, and a field or a position of a variant with its variant.

A class on the list that the JDK does not have is not refused, because no JDK is read.
An `extern` naming a member the JVM does not have is a class file the JVM refuses to link, which
is the author's claim failing rather than a program's.

## Properties

These hold and are checked by drawn properties in the runner:

1. Every `extern` declaration round trips: printed and parsed again, it is the same declaration.
2. A signature holding a type that does not cross is `L0425`, whichever position it is in.
3. A call of an `extern` is lowered reaching only the class that declaration names.
4. A body guarded by a `Result` leaves one on the stack down every path out of it.
5. No `extern` declaration writes a body, so no two of them can disagree about one member.
6. A declaration written `int` reaches the member for an `int` and leaves a `long` behind it.
7. A `method` on a type written `interface` is lowered to the call the JVM makes on one.
8. A `List<T>` is accepted where a member takes one, and refused where a member gives one back.
9. A declaration written `char` reaches the member for a `char` and leaves a `long` behind it.
10. A declaration that narrows a parameter answers `None` for every argument outside the `int`
    range, and reaches its member for none of them.
11. Every class that the tree names in an `extern` is accepted.
12. A drawn class on no list is `L0434`, and an archive accounts for it outside the JDK packages.
13. An extern type drawn into a result, a field, a variant, or a call is `L0435` naming that place.
14. A drawn program that only reaches, binds, and passes a foreign reference compiles.

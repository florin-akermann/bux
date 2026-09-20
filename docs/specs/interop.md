# Reaching a Java class

An `extern` declaration names one member of one Java class and gives it a Lumen signature.

## Intent

`docs/design.md` section 17 states the boundary, and `docs/implementation.md` section 3 asks for
it.
`io` and `files` were the compiler's own modules because each reaches a JVM method no Lumen source
could name, and they are library modules written over these declarations once one can.
This spec says what an `extern` declaration is, what crosses the boundary, and what is refused.

The boundary is a signature and nothing more.
Java's object model does not come with it: there is no subtyping, no widening, no overload to
pick between, no class hierarchy, and no `equals`, `hashCode`, or `toString` reaching a value.
What a declaration cannot say is not reachable, and the answer to that is another declaration.

## What a declaration is

There is one form per kind of member the JVM has, and no sixth kind.

```text
extern type File = "java.io.File"
extern type Path = "java.nio.file.Path"
extern type PrintStream = "java.io.PrintStream"

extern field out() -> PrintStream = "java.lang.System.out"
extern static read_string(path: Path) -> Result<String, String> = "java.nio.file.Files.readString"
extern method to_path(file: File) -> Path = "toPath"
extern new opened(name: String) -> File
```

`type` names a class, under a Lumen type name that values of it are held as.
`field` names a static field, and reading it is a call of no arguments.
`static` names a static method.
`method` names an instance method of a class, whose receiver is the first parameter.
`new` names a constructor, of the class its result is.

A declaration says which kind it is, so nothing about a call is worked out from the shape of a
signature.
`field` and `static` name the class as well as the member, because neither has a receiver to name
it.
`method` names the member alone, because the receiver's type already says which class it is on,
and `new` names neither, because its result already says.
Each of those is a disagreement that cannot be written rather than one that is reported.

Everything else about an `extern` function is an ordinary function of the module that declares it.
It is reached through that module's name, it is part of the module's surface, and a call of it is
a call, which `docs/specs/calls.md` states.

## What crosses

A parameter and a result are Lumen types, and each compiles to exactly the JVM type the member's
own descriptor names.

| Lumen         | JVM                     | where          |
|---------------|-------------------------|----------------|
| `Bool`        | `boolean`               | either         |
| `Int`         | `long`                  | either         |
| `String`      | `java.lang.String`      | either         |
| an extern type | the class it names      | either         |
| `()`          | `void`                  | a result       |
| `Option<T>`   | what `T` is, or `null`  | a result       |
| `Result<T, String>` | what `T` is, or thrown | a result |

Nothing else crosses.
A `List`, a record, a variant, and a type parameter are each a Lumen type a Java member has no
descriptor for, and an `extern` naming one is `L0425`.
The JVM's `int`, `double`, and the rest of its primitives are types no Lumen type compiles to, so
a member whose descriptor names one is not reachable and the answer is to name one that does not.

An extern type is a type like any other from where a program stands.
It is held, handed on, given back, and matched by nothing, because it declares no variants and no
fields.
It has no identity a program can reach, as every Lumen type has none.
It is compared, hashed, or shown only where an `instance` written over `extern` declarations says
how, and a `derive` naming one is `L0427`: a derive reads what a type holds, and what this one
holds is the JVM's.

## The two mappings

`Option<T>` and `Result<T, String>` are what the two things Java gives back that Lumen has no
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
`Ok` and `Some` each carry one value, and a member that gives nothing back leaves none to carry,
so `Result<(), String>` and `Option<()>` are each `L0425` and the answer is a member that gives
something back.
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

Nothing of this is visible from Lumen.
A program that writes `io.println` never learns what carries it, exactly as before.

A `method` is called the way an instance method of a class is called, and an interface's method
is called another way.
Which of the two a Java name is is written in that name's own class file, and the compiler reads
none, so an `extern type` naming an interface and a `method` taking one as its receiver compiles
to a call the JVM refuses to link.
That is the author's claim failing in the same way naming a member the JVM does not have is, and
reaching an interface waits for the declaration that says it is one.

## The errors

| name                  | code    | message                                                    |
|-----------------------|---------|------------------------------------------------------------|
| type does not cross   | `L0425` | `List<Int>` is no type a Java member takes                 |
| not a Java name       | `L0426` | `java..File` is no Java name                               |
| derive of an extern type | `L0427` | `File` is an extern type, and a derive reads what a type holds |
| no class to reach     | `L0428` | a `method` reaches a class, and this signature names none  |

`L0425` helps with ``a boundary carries `Bool`, `Int`, `String`, and a type an `extern` names``.
It points at the type as the signature writes it, and is raised for a result as well as for a
parameter, with `()`, `Option`, and `Result` accepted only as a result.
The message says what the member does with a value written there — takes it, gives it back, or,
for a `field`, holds it — because one signature can write all three.

`L0426` helps with `a Java name is its segments, each a name, with a dot between two of them`.
A name is refused where a segment is empty, where it begins with a digit, or where it holds
anything but letters, digits, `_`, and `$`.
A `field` and a `static` name at least two segments, because the last is the member and the rest
is the class; a `type` names at least one; a `method` names exactly one.

`L0427` helps with ``write an `instance` over `extern` declarations instead``.

`L0428` helps with ``a class is `String`, or a type an `extern type` declares``.
A `method` reads its class off its first parameter and a `new` off its result, so each of those
has to name one; a signature that names none leaves the declaration with no class to reach, and
that is refused where it is written rather than met as a state the lowering has no answer for.

A Java class or member that is not there is not refused, because nothing is loaded to refuse it
against: `docs/specs/library.md` states that the library is read with no classpath and no JDK.
An `extern` naming a member the JVM does not have is a class file the JVM refuses to link, which
is the author's claim failing rather than a program's.

## Properties

These hold and are checked with property-based tests:

1. Every `extern` declaration round trips: printed and parsed again, it is the same declaration.
2. A signature holding a type that does not cross is `L0425`, whichever position it is in.
3. A call of an `extern` is lowered reaching only the class that declaration names.
4. A body guarded by a `Result` leaves one on the stack down every path out of it.
5. No `extern` declaration writes a body, so no two of them can disagree about one member.

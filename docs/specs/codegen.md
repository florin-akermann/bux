# Code generation

## Intent

A checked module becomes class files a JVM can load.
The phase consumes the typed tree, lowers it to a JVM intermediate representation, and writes the
class files that representation describes.
`docs/implementation.md` section 1 sets the target: the class-file version is the current JDK's,
and no older JVM is supported.

Nothing in this spec is visible from Lumen.
A reader of a Lumen program never needs it, and no diagnostic mentions a class, a descriptor, or a
stack frame.
It is written down because the output is a contract with the JVM, and because two compilations of
one source must produce the same bytes.

## What is written

`lumen build <file>` writes class files beside the source file.
A module is one source file, so `demo.lm` yields these:

```text
demo.class          the module: one static method per function
demo/User.class     one class per type the module declares
lumen/Option.class  the prelude types, which every module may reach
```

A type declared in the module is a class in a package named after the module.
A variant of an algebraic data type is a class of that package too, named after the variant,
which name resolution has already made unique within the module.

The prelude types are written on every build, in the package `lumen`.
They are `Option` with `Some` and `None`, and `Result` with `Ok` and `Err`.
Writing them with the module keeps a build self-contained: there is no runtime jar to install and
no version of one to agree with.

## What a value is

A Lumen type is carried by a JVM type:

```text
Int             long
Bool            boolean
String          java.lang.String
a declared type the class written for it
Option<T>       lumen.Option
Result<T, E>    lumen.Result
List<T>         java.util.List
a type parameter java.lang.Object
```

`Int` is `long` because `Int` is 64 bits wide, which `docs/design.md` section 3 states.

`()` is carried by nothing at all.
A function whose result is `()` returns `void`, and a binding of a unit value occupies no local.
Version 0.1 can write `()` but can do nothing with one, so a representation would never be read.

A type parameter erases to `java.lang.Object`, so a call that passes an `Int` or a `Bool` where a
parameter is a type parameter boxes it, and one that reads such a result back at a settled type
unboxes it.
Boxing is decided from the types inference gave the call, never from the shape of the value.

## How a function is called

Each function of the module is a `public static` method of the module class, named as it is
written, with the descriptor its signature gives.
A module declares each name once, so no two methods share a name and nothing is overloaded.

A function is reached by `invokestatic` on the module class.
Version 0.1 has no function values, so nothing else calls one.

## How a type is laid out

A record type is a `final` class with one `final` field per field it declares, in the order it
declares them, and one constructor taking them in that order.
`user.name` is a `getfield`, and `user { active: false }` is a new instance built from the fields
of the old one.

An algebraic data type is an `abstract` class carrying one `final int` field, `tag`.
Each variant is a `final` class extending it, whose constructor passes the variant's position in
the declaration as the tag, and which declares one `final` field per value the variant carries.
A variant that carries values positionally names them `value0`, `value1`, and so on.

`match` reads the tag and switches on it.
An arm that binds what a variant carries casts to the variant's class and reads its fields.
A `match` on a `Bool` switches on the value, and one on an `Int` or a `String` compares.

## What the bytes look like

The class-file version is the current JDK's, and the minor version is 0.
Every method carries a `Code` attribute, and every `Code` attribute that branches carries a
`StackMapTable`, which the verifier requires.

The output is byte-reproducible.
Two compilations of one source produce identical files, so nothing in the writer depends on a
timestamp, a build path, or the order a hash map iterates.
The constant pool is built in the order entries are first asked for, which the lowering fixes.

## The errors

Code generation raises no diagnostic.
Every way a program can be wrong has been refused by an earlier phase, and a module that reaches
this phase is written out.

## Properties

These hold and are checked with property-based tests:

1. Lowering and writing a checked module never panics and is deterministic.
2. Compiling one source twice gives byte-identical class files.
3. Every class written begins with the class-file magic and the current version.
4. Every method a module declares is written as a static method of the module class.
5. Every constant pool entry a method refers to is within the pool.

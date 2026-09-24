Compile a source file to the class files a JVM loads.

`bux build` runs the whole compiler over a file and writes the result under `target/`: beside the
manifest of its package, or beside the file where it is in no package. Everything `bux check`
reports is reported here too, in the same layout, and nothing is written when the compiler
refuses the program.

Every class of a build lands in that one `target/` directory, and no class lands beside a source.
Every module of a package, in `src/` or in `tests/`, writes into the one `target/` of the package.
A clean build is `rm -rf target/`, because no other directory holds what a build writes.

A module is one source file, so `demo.bx` becomes `target/demo.class`, holding one `public static`
method per function the module declares. Each type the module declares becomes a class of its
own in a package named after the module, so `type User` becomes `target/demo/User.class`. A
variant of an algebraic data type becomes a class beside its type, named after both.

A program of several modules is several classes. Every module the file imports is compiled and
written too, so `import greeting` puts `target/greeting.class` beside `target/demo.class`, and a
call of `greeting.hello` is a call of a static method of that class. A module beside the file that
nothing imports is not read and not written. A module of the library is written in the package
`bux/library`, so `import strings` puts `target/bux/library/strings.class` there, and a file of
the program may have the name of a library module.

The types the prelude supplies are written on every build, in the package `bux` under
`target/`. There is no runtime to install and no version of one to agree with: a build is
self-contained.

The output is reproducible. Two builds of one unchanged file write identical bytes, so nothing in
them depends on a timestamp, on where the build ran, or on the order a hash map iterated.

The class-file version is the current JDK's, which is the only one Bux targets. A JDK is needed
to run what is written, not to write it.

A module is named after its file, so the file's name has to be one a class may have: a name
holding `.`, `;`, `[` or `/` is refused rather than written out as a class no JVM would load.

A build reads the `jar` lines of the manifest too, which `bux help packages` states. Each archive
is there, has the hash that its line states, is a Java archive, and has no `Class-Path`, or the
build is refused against its line before a class is written. The archive is not copied.

A hole is refused here. `todo("a reason")` is what an unfinished body is written as, and a hole
has nothing to run, so there is nothing for a build to write. Every hole in the module is named
rather than the first, because a build is how a reader learns what is left; nothing is written
when a module holds one. `bux check` is the command that accepts a hole.

Exit codes: 0 when the class files are written, 1 when the compiler refuses the program, and 2
when a file cannot be read or written, or is named something no class can be called.

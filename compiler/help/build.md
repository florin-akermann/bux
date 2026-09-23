Compile a source file to the class files a JVM loads.

`lumen build` runs the whole compiler over a file and writes the result beside it. Everything
`lumen check` reports is reported here too, in the same layout, and nothing is written when the
compiler refuses the program.

A module is one source file, so `demo.lm` becomes `demo.class`, holding one `public static`
method per function the module declares. Each type the module declares becomes a class of its
own in a package named after the module, so `type User` becomes `demo/User.class`. A variant of
an algebraic data type becomes a class beside its type, named after both.

A program of several modules is several classes. Every module the file imports is compiled and
written too, so `import greeting` puts `greeting.class` beside `demo.class`, and a call of
`greeting.hello` is a call of a static method of that class. A module beside the file that
nothing imports is not read and not written.

The types the prelude supplies are written on every build, in the package `lumen`. There is no
runtime to install and no version of one to agree with: a build is self-contained.

The output is reproducible. Two builds of one unchanged file write identical bytes, so nothing in
them depends on a timestamp, on where the build ran, or on the order a hash map iterated.

The class-file version is the current JDK's, which is the only one Lumen targets. A JDK is needed
to run what is written, not to write it.

A module is named after its file, so the file's name has to be one a class may have: a name
holding `.`, `;`, `[` or `/` is refused rather than written out as a class no JVM would load.

A hole is refused here. `todo("a reason")` is what an unfinished body is written as, and a hole
has nothing to run, so there is nothing for a build to write. Every hole in the module is named
rather than the first, because a build is how a reader learns what is left; nothing is written
when a module holds one. `lumen check` is the command that accepts a hole.

Exit codes: 0 when the class files are written, 1 when the compiler refuses the program, and 2
when a file cannot be read or written, or is named something no class can be called.

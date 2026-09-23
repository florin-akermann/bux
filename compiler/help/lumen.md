Lumen is a small, statically typed language that compiles to JVM bytecode.
Go's simplicity. Haskell's types. The JVM's runtime. A compiler written in itself.

Source that is not in canonical form does not compile: run `lumen fmt` first.

Commands:
  fmt       Rewrite a source file in canonical form
  check     Report the first thing about a source file or a package the compiler will not have
  build     Compile a source file or a package to the class files a JVM loads
  run       Compile a source file and run the program it holds
  test      Run the examples a module states about its functions
  api       Print the public surface of a module
  explain   Print the long form of one diagnostic code

Run `lumen help <command>` for the long form of any of them.

A package is a directory of modules with a `bux.package` manifest beside them; `check` and
`build` each take one. `lumen help check` says what the manifest states and where an import
looks.

Every error carries a code on its first line; `lumen explain <code>` says more about it.

A process owns a state and takes messages; `lumen help process` says how one is written.

The same command line is written in Bux, in `compiler/`. Build it with
`lumen build compiler/main.lm`, then start it with `bin/bux`, on the JDK that JAVA_HOME names.

Further commands land one at a time; each is documented here in the change that adds it.

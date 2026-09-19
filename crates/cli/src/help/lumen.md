Lumen is a small, statically typed language that compiles to JVM bytecode.
Go's simplicity. Haskell's types. The JVM's runtime. A compiler written in Rust.

Source that is not in canonical form does not compile: run `lumen fmt` first.

Commands:
  fmt       Rewrite a source file in canonical form
  check     Report the first thing about a source file the compiler will not have
  explain   Print the long form of one diagnostic code

Run `lumen help <command>` for the long form of any of them.

Every error carries a code on its first line; `lumen explain <code>` says more about it.

Further commands land one at a time; each is documented here in the change that adds it.

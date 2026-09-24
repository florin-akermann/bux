# Processes and messages

A process owns a state, and a message is the only way to that state.
This spec states `process`, `spawn`, the handle, the mailbox, `send`, and `ended`.
`docs/design.md` section 15 gives the reasons, and this spec gives the rules.

## Intent

The compiler and `bin/runner` do a lot of independent work, and a machine has many processors.
A program uses them through processes and messages, and never through a lock or a shared value.
One process has one shape, so a reader who has seen one process can read every process.
A message that cannot arrive is a typed answer, and a full mailbox makes the sender wait.

## A process

A module declares a process with `process`, a name, and two functions.

```text
process Counter {
    fn start(initial: Int) -> Int {
        initial
    }

    fn receive(total: Int, message: Counted) -> Next<Int> {
        match message {
            Add(amount) => Continue(total + amount)
            Stop => Done
        }
    }
}
```

The first function is `start`, and the second is `receive`.
A process declares nothing else.
`start` takes the values that `spawn` is given, and gives the first state.
`receive` takes the state and one message, and gives `Next` of the state.
The prelude declares `Next<T>` as `Continue(T)` or `Done`.
`Continue(state)` keeps the process, and `receive` takes the next message with that state.
`Done` ends the process, and the state that `receive` took is its last state.

The type of the state is what `start` gives.
The type of the message is the second parameter of `receive`.
One process has one type of each, so a message is an ADT with one variant for each kind.

`process` and `spawn` are keywords, so no name in a program is `process` or `spawn`.
A process sits among the items of a module, in the order `docs/design.md` section 13 gives.
It sits below the first function that spawns it, as a function sits below its first caller.

## The shape

The compiler holds the shape before it infers a type, so each break has its own code.

- `L0802`: the first function is not `start`.
- `L0803`: the second function is not `receive`.
- `L0804`: a function follows `receive`.
- `L0805`: `start` or `receive` declares a type parameter.
- `L0806`: `start` or `receive` leaves the type of a parameter or of its result unwritten.
- `L0807`: `receive` does not take two values.
- `L0808`: the body of `receive` is not one `match` on its message parameter.
- `L0809`: an arm of that `match` is not one call or one name.
- `L0810`: `receive` does not take the state `start` gives, or does not give `Next` of it.
- `L0811`: what `start` takes, the state, or the message is or holds an `extern` type.

A process holds values only, and a value of an `extern` type is a foreign reference.
A foreign reference has identity and mutation, so it stays with the function that reaches it.
`L0811` reads what `start` takes on its own, because `start` need not keep it in the state.
The message names the process and the type, and the check follows each type into what it holds.

Nothing in the source calls `start` or `receive`, so inference has no call to fill a type from.
Each type is therefore written, and no function of a process is generic.
A function of a process states no example, because no expression can call it.

An arm that is one call or one name can call a function of the module with the work in it.
`Add(amount) => added(total, amount)` is such an arm, and `added` gives `Next<Int>`.
The rule forces the work out of the branch, and the name of the work into the source.

## Spawn

`spawn` starts a process and gives its handle.

```text
let counting = spawn Counter(10)
```

`spawn` is followed by a call of a process.
The arguments are the values `start` takes, and each is passed by value.
`start` runs on the new process, so `spawn` never waits for it.

The arguments follow the rules of a call to `start`, so they are named where two share a type.
`spawn` of anything else is refused as `L0800`, and so is a process name with no call after it.
A process named without `spawn`, as a call or as a value, is refused as `L0801`.

A module offers each process it declares, and a module that imports it reaches one by its name.
`spawn ticks.Ticker(5)` starts `Ticker` of the module `ticks`, as `ticks.count(5)` calls a function.
The arguments follow the rules of a call through a module, so they are passed in order.
`L0800` refuses `spawn ticks.count(5)`, and `L0801` refuses `ticks.Ticker(5)` without `spawn`.
A process is reached through a module name only, because no value holds a process.

## The handle

The handle of a process is `Process<Message, State>`.
It is the one name the rest of the program has for the process.
A second `spawn` of the same process gives a second handle, to a second process.

A handle has identity, so it has no `Eq`, no `Ord`, no `Hash`, and no `Show`.
No type that holds a handle can derive one of them.
A handle crosses no `extern` boundary, because a Java method has no use for one.

A handle is a value in every other way.
A message can carry one, and that is how a process gets an address to answer on.

```text
type Asked =
    | Get(Process<Answer, Int>)
```

A requester that is itself a process puts its own handle in the request.
There is no reply primitive, because a reply is one more message.

## The mailbox

Each process has one mailbox, and the mailbox holds 64 messages at most.
The bound is the same for every mailbox, and no program can change it.
`receive` takes the messages in the order the mailbox took them.
A message that one sender sends before another is taken before it.

## Send

`send(to, message, waiting)` puts `message` in the mailbox of `to`, and gives a `Sent`.

```text
type Sent =
    | Delivered
    | MailboxFull
    | ProcessEnded

type Waiting =
    | NoWait
    | Milliseconds(Int)
    | NoLimit
```

The prelude declares both types.
`waiting` says how long `send` waits while the mailbox is full.
`NoWait` does not wait, `Milliseconds(n)` waits `n` milliseconds at most, and `NoLimit` waits.

- `Delivered`: the mailbox took the message.
- `MailboxFull`: the mailbox was still full when the wait ended.
- `ProcessEnded`: the process had ended, so no mailbox takes the message.

A `send` to a process that has ended never waits, whatever `waiting` says.
A `send` with `NoLimit` to a full mailbox waits until there is room or until the process ends.
A process can end while a message waits in its mailbox, and that message is not read.
`Delivered` says that the mailbox took the message, and not that `receive` read it.

A `send` gives a `Sent`, and a program that does not read it writes `_ =`.
That is the rule of `docs/specs/discarding.md`, and it applies here as it does everywhere.

## Ended

`ended(handle)` waits until the process ends, and gives its last state.
The last state is the state that `receive` took with the message it answered with `Done`.
`ended` on a process that never gets a message that ends it never returns.
`ended` twice on one handle gives the same state twice.

A JVM error, such as a stack overflow, can stop `start` or `receive` part of the way through.
Then the process ends, and the error goes to standard error.
A `send` to it gives `ProcessEnded`, and `ended` on it stops the caller with the same error.
So no one waits for a process that the error stopped.

## Waking and time

A process never wakes without a message.
`receive` has no deadline, and there is no `after`.
No timer and no sleep is part of the language.
A process that must act at a time is sent a message at that time by another process.

## Running on the JVM

Each process runs on a JVM virtual thread, and a blocked process costs no platform thread.
The mailbox is a `java.util.concurrent.ArrayBlockingQueue` of capacity 64.
The end of a process is a `java.util.concurrent.CompletableFuture`, which `ended` joins.
Every class used is in `java.base`.
No program can see a thread, a queue, or a future, and a program names none of them.

The class of a process is `<module>/process$<Name>`, which no type of a program can be named.
`start` and `receive` are the static methods `<Name>$start` and `<Name>$receive` of the module.
The handle is the class `bux/Process`, and each module that declares a process writes it.
Each writes the same bytes, as each module writes the same class of a list.

## Properties

`tests/spawned.bx` holds these properties over drawn programs and drawn messages.

1. A mailbox never holds more than 64 messages.
2. The messages one sender sends before the message that ends a process arrive in order.
3. `ended` gives the state that `receive` held when it gave `Done`.
4. The shape check accepts every well-formed drawn process, and refuses each drawn break.

`tests/spec/concurrency/` holds a counter, a worker pool, a full mailbox, and each refused shape.
`tests/spec/concurrency/elsewhere/` spawns a process that another module declares.

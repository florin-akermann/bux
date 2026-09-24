Processes
---------

A process owns a state, and a message is the only way to that state. A module declares a process
with `process`, a name, and two functions: `start`, then `receive`, and nothing else.

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

`start` takes the values `spawn` is given and gives the first state. `receive` takes the state
and one message, and gives `Continue` of the next state or `Done`, which ends the process. Every
type is written, and neither function is generic. The body of `receive` is one `match` on the
message, and each arm is one call or one name; other work goes in a function the arm calls.
Each break of the shape has its own code, `L0802` to `L0810`, and `bux explain` says more.

`spawn Counter(10)` starts a process on a JVM virtual thread and gives its handle, a
`Process<Counted, Int>`: what it accepts, then its state. A handle has no `Eq` and crosses no
`extern`. A message can carry one, which is how a process is given an address to answer on.

A process holds values only: what `start` takes, the state, and the message hold no `extern`
type, which `L0811` refuses. A foreign reference stays with the function that reaches it.

`send(counting, Add(3), NoLimit)` puts a message in the mailbox and gives a `Sent`: `Delivered`,
`MailboxFull`, or `ProcessEnded`. A mailbox holds 64 messages. While it is full, `send` waits as
the third argument says: `NoWait`, `Milliseconds(n)`, or `NoLimit`. A process that has ended
takes nothing, and `send` says so without a wait.

`ended(counting)` waits for the process to end and gives its last state. A process wakes only for
a message: there is no timeout on `receive`, no timer, and no sleep.

# Ideas for compiler

Assorted ideas

## Types

It's not possible to define neat types of builtin commands like `times`, `...`,
or `do!`. There are at least a few challenges:

1. The exact effect on a stack may be impossible to exactly represent. An easy
  example is a value coming from standard input, that influences an argument to
  one of these. The same goes for any IO.
2. Some words allow for side effects like dynamically defining new words. Again
  this is complicated if input is coming from standard input, or some other IO
  unknown until runtime.

First, stack issues:

While it's not possible to _know_ an exact stack effect, it appears trivial to
me to know failure cases. `[pop pl] 3 times` is safe to perform when the
immediately preceding type is a quote of size 3 or greater, has a side effect
of printing 3 lines, and will result in a quote reduced on the right in size by
three items.

What I think this means is that `times` (and other higher-order procedures) can
be represented as a **lazy type** that can be filled in either at compile time
and completely reduced, or can be represented as a range of possibilities when
an argument is unknown.

Like Haskell, we can know and trace what is from IO as an unknown value.
Anything else we can opportunistically treat as a Zig-like `comptime` value. I
believe the latter should open up many compiler optimizations that may not be
possible in other programming paradigms.

We can also statically analyze a program's unknown values to know that, for
example, the exact program `stdin ... [pl] 5 times` will succeed "perfectly"
for 5 lines of standard input, succeed with "waste" for more lines, and result
in stack underflow for less lines.

Exactly how to represent exceptional code (Result types, try/catch with
exceptions, halt and catch fire...) can be decided independently.

_P.S. a problem arises with a program like `stdin pop quote do`

Second, side effects:

Types can know when they have side effects, and even propagate them. Again
there should be a way to understand both potential side effects that are
unknown-from-IO vs known-at-compile-time.

TODO: this could be expanded.

Lastly:

I believe this distinction between unknown (IO) and known (comptime) can be
modeled in a type system. The types can start out lazily (when implemented in
a type system) and can be resolved to more specific types during interpretation
or compilation.

## No dynamics in compiled mode?

It will be useful to have a compiled mode that disallows the ability to
execute or define unknown code. In fact, this should probably be the default
"safe" mode of compilation. When interpreted, this is worth a warning.

## Represent all subprocedures as jump addresses

GOTO will never die. Calling a function in a compiled program should not
require a string equality lookup, and not even require a lookup table. Like an
index map, I believe we can represent program state, including enough
information to (for example) create a stack trace, independently from the flow
of control.


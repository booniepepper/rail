# Symbol table

Introduced as `stab` in 0.15.0 a simple key-value map data structure. Backed by
Rust stdlib's HashMap.

# Definition table

Idea (as JSON)

```
{
    "key-1": ["first"],
    "key-2": ["second", "first"],
}
```

`get` retrieves the latest value for a given key.

`set` pushes a value onto a key's stack.

Like anything... It's immutable if you don't mutate it.

Side note: Make `push`/`peek`/`pop` or `enq`/`deq` polymorphic semantics for access?

The structure is (loosely) inspired by Forth's dictionary stack. It could be
implemented as a single stack, but for a map with many keys it would be more
efficient to implement as a hash map of stacks.

# Any type maps

See:

* JavaScript "Object Literal"
* Python `dict`
* Ruby Hash
* Clojure map and https://youtu.be/2V1FtfBDsLU (esp 27:19 and following)

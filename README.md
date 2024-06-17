# minimizer-iter

[![crates.io](https://img.shields.io/crates/v/minimizer-iter)](https://crates.io/crates/minimizer-iter)
[![docs](https://img.shields.io/docsrs/minimizer-iter)](https://docs.rs/minimizer-iter)

Iterate over minimizers of a DNA sequence.

## Features

- iterates over minimizers in a single pass
- yields bitpacked minimizers with their position
- supports [mod-minimizers](https://doi.org/10.1101/2024.05.25.595898), introduced by Groot Koerkamp & Pibiri
- supports custom bit encoding of the nucleotides
- supports custom [hasher](https://doc.rust-lang.org/stable/core/hash/trait.BuildHasher.html), using [wyhash](https://github.com/JackThomson2/wyhash2) by default
- can be seeded to produce a different ordering

If you'd like to use the underlying data structure manually, please have a look at the [minimizer-queue](https://github.com/imartayan/minimizer-queue) crate instead.

## Example usage

```rust
use minimizer_iter::MinimizerBuilder;

// Build an iterator over minimizers
// of size 5 with a window of size 4
// for the sequence "TGATTGCACAATC"
let min_iter = MinimizerBuilder::<u64>::new()
    .minimizer_size(5)
    .width(4)
    .iter(b"TGATTGCACAATC");

for (minimizer, position) in min_iter {
    // ...
}
```

If you'd like to use mod-minimizers instead, just change `new()` to `new_mod()`:
```rust
use minimizer_iter::MinimizerBuilder;

// Build an iterator over mod-minimizers
// of size 5 with a window of size 4
// for the sequence "TGATTGCACAATC"
let min_iter = MinimizerBuilder::<u64, _>::new_mod()
    .minimizer_size(5)
    .width(4)
    .iter(b"TGATTGCACAATC");

for (minimizer, position) in min_iter {
    // ...
}
```

If you need longer minimizers (> 32 bases), you can specify a bigger integer type such as `u128`:
```rust
MinimizerBuilder::<u128>::new()
    .minimizer_size(...)
    .width(...)
    .iter(...)
```

See the [documentation](https://docs.rs/minimizer-iter) for more details.

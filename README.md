# minimizer-iter

[![crates.io](https://img.shields.io/crates/v/minimizer-iter)](https://crates.io/crates/minimizer-iter)
[![docs](https://img.shields.io/docsrs/minimizer-iter)](https://docs.rs/minimizer-iter)

Iterate over minimizers of a DNA sequence.

## Features

- iterates over minimizers in a single pass
- yields bitpacked minimizers with their position
- supports custom bit encoding of the nucleotides
- supports custom [hasher](https://doc.rust-lang.org/stable/core/hash/trait.BuildHasher.html), using [wyhash](https://github.com/JackThomson2/wyhash2) by default
- can be seeded to produce a different ordering

## Example usage

```rust
use minimizer_iter::MinimizerBuilder;

// Build an iterator over minimizers
// of size 3 with a window of size 4
// for the sequence "TGATTGCACAATC"
let min_iter = MinimizerBuilder::<u64>::new()
    .minimizer_size(3)
    .width(4)
    .iter(b"TGATTGCACAATC");

for (minimizer, position) in min_iter {
    // ...
}
```

If you'd like to have longer minimizers (up to 64 bases), you can use `u128` instead:
```rust
MinimizerBuilder::<u128>::new()
    .minimizer_size(...)
    .width(...)
    .iter(...)
```

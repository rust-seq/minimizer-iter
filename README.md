# minimizer-iter

[![crates.io](https://img.shields.io/crates/v/minimizer-iter)](https://crates.io/crates/minimizer-iter)
[![docs](https://img.shields.io/docsrs/minimizer-iter)](https://docs.rs/minimizer-iter)

Iterate over minimizers of a DNA sequence.

## Features

- iterates over minimizers in a single pass
- yields bitpacked minimizers with their position
- supports [mod-minimizers](https://doi.org/10.1101/2024.05.25.595898), introduced by Groot Koerkamp & Pibiri
- supports canonical minimizers
- supports custom bit encoding of the nucleotides
- supports custom [hasher](https://doc.rust-lang.org/stable/core/hash/trait.BuildHasher.html), using [wyhash](https://github.com/JackThomson2/wyhash2) by default
- can be seeded to produce a different ordering

If you'd like to use the underlying data structure manually, have a look at the [minimizer-queue](https://github.com/rust-seq/minimizer-queue) crate.

## Example usage

```rust
use minimizer_iter::MinimizerBuilder;

// Build an iterator over minimizers
// of size 21 with a window of size 11
// for the sequence "TGATTGCACAATC"
let min_iter = MinimizerBuilder::<u64>::new()
    .minimizer_size(21)
    .width(11)
    .iter(b"TGATTGCACAATC");

for (minimizer, position) in min_iter {
    // ...
}
```

If you'd like to use mod-minimizers instead, just change `new()` to `new_mod()`:
```rust
use minimizer_iter::MinimizerBuilder;

// Build an iterator over mod-minimizers
// of size 21 with a window of size 11
// for the sequence "TGATTGCACAATC"
let min_iter = MinimizerBuilder::<u64, _>::new_mod()
    .minimizer_size(21)
    .width(11)
    .iter(b"TGATTGCACAATC");

for (minimizer, position) in min_iter {
    // ...
}
```

Additionally, the iterator can produce canonical minimizers so that a sequence and its reverse complement will select the same minimizers.
To do so, just add `.canonical()` to the builder:
```rust
MinimizerBuilder::<u64>::new()
    .canonical()
    .minimizer_size(...)
    .width(...)
    .iter(...)
```

If you need longer minimizers (> 32 bases), you can specify a bigger integer type such as `u128`:
```rust
MinimizerBuilder::<u128>::new()
    .minimizer_size(...)
    .width(...)
    .iter(...)
```

See the [documentation](https://docs.rs/minimizer-iter) for more details.

## Benchmarks

To run benchmarks against other implementations of minimizers, clone this repository and run:
```sh
cargo bench
```

## Contributors

- [Igor Martayan](https://github.com/imartayan) (main developer)
- [Pierre Marijon](https://github.com/natir)

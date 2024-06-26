use crate::algorithm::{Minimizer, MinimizerAlgorithm, ModMinimizer};
use crate::iterator::*;
use core::hash::{BuildHasher, Hash};
use core::marker::PhantomData;
use minimizer_queue::DefaultHashBuilder;
use num_traits::PrimInt;

/// A builder for iterators over minimizers.
///
/// # Examples
///
/// ```
/// use minimizer_iter::MinimizerBuilder;
///
/// // Build an iterator over minimizers
/// // of size 3 with a window of size 4
/// // for the sequence "TGATTGCACAATC"
/// let min_iter = MinimizerBuilder::<u64>::new()
///     .minimizer_size(3)
///     .width(4)
///     .iter(b"TGATTGCACAATC");
///
/// for (minimizer, position) in min_iter {
///     // ...
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinimizerBuilder<
    T: PrimInt = u64,
    A: MinimizerAlgorithm = Minimizer,
    S: BuildHasher = DefaultHashBuilder,
    const CANONICAL: bool = false,
> {
    minimizer_size: usize,
    width: u16,
    hasher: S,
    encoding: [u8; 256],
    _marker: PhantomData<(T, A)>,
}

impl<T: PrimInt + Hash> MinimizerBuilder<T> {
    /// Sets up the `MinimizerBuilder` with default values:
    /// - minimizer_size = 21
    /// - width = 11 (31 - 21 + 1)
    /// - hasher = [`DefaultHashBuilder`]
    /// - encoding: A = `00`, C = `01`, G = `10`, T = `11`
    #[inline]
    pub fn new() -> Self {
        Self::_new()
    }
}

impl<T: PrimInt + Hash> Default for MinimizerBuilder<T> {
    #[inline]
    fn default() -> Self {
        Self::_new()
    }
}

impl<T: PrimInt + Hash, S: BuildHasher> MinimizerBuilder<T, Minimizer, S, false> {
    /// Builds an iterator over the minimizers and their positions in the given sequence.
    #[inline]
    pub fn iter(self, seq: &[u8]) -> MinimizerIterator<T, S> {
        MinimizerIterator::new(
            seq,
            self.minimizer_size,
            self.width,
            self.hasher,
            self.encoding,
        )
    }

    /// Builds an iterator over the positions of the minimizers in the given sequence.
    #[inline]
    pub fn iter_pos(self, seq: &[u8]) -> MinimizerPosIterator<T, S> {
        MinimizerPosIterator::new(
            seq,
            self.minimizer_size,
            self.width,
            self.hasher,
            self.encoding,
        )
    }
}

impl<T: PrimInt + Hash, S: BuildHasher> MinimizerBuilder<T, Minimizer, S, true> {
    /// Builds an iterator over the canonical minimizers and their positions in the given sequence with a boolean indicating a reverse complement.
    /// It requires an odd width to break ties between multiple minimizers.
    #[inline]
    pub fn iter(self, seq: &[u8]) -> CanonicalMinimizerIterator<T, S> {
        assert_eq!(
            self.width % 2,
            1,
            "width must be odd to break ties between multiple minimizers"
        );
        CanonicalMinimizerIterator::new(
            seq,
            self.minimizer_size,
            self.width,
            self.hasher,
            self.encoding,
        )
    }

    /// Builds an iterator over the positions of the canonical minimizers in the given sequence with a boolean indicating a reverse complement.
    /// It requires an odd width to break ties between multiple minimizers.
    #[inline]
    pub fn iter_pos(self, seq: &[u8]) -> CanonicalMinimizerPosIterator<T, S> {
        assert_eq!(
            self.width % 2,
            1,
            "width must be odd to break ties between multiple minimizers"
        );
        CanonicalMinimizerPosIterator::new(
            seq,
            self.minimizer_size,
            self.width,
            self.hasher,
            self.encoding,
        )
    }
}

const R: usize = 4;

impl<T: PrimInt + Hash> MinimizerBuilder<T, ModMinimizer> {
    /// Sets up the `MinimizerBuilder` for mod-minimizers with default values:
    /// - minimizer_size = 21
    /// - width = 11 (31 - 21 + 1)
    /// - hasher = [`DefaultHashBuilder`]
    /// - encoding: A = `00`, C = `01`, G = `10`, T = `11`
    #[inline]
    pub fn new_mod() -> Self {
        Self::_new()
    }
}

impl<T: PrimInt + Hash, S: BuildHasher> MinimizerBuilder<T, ModMinimizer, S, false> {
    /// Builds an iterator over the mod-minimizers and their positions in the given sequence.
    #[inline]
    pub fn iter(self, seq: &[u8]) -> ModSamplingIterator<T, S> {
        assert!(
            self.minimizer_size >= R,
            "mod-minimizers require minimizer_size ≥ r={R}"
        );
        ModSamplingIterator::new(
            seq,
            self.minimizer_size,
            self.width,
            R + ((self.minimizer_size - R) % self.width as usize),
            self.hasher,
            self.encoding,
        )
    }

    /// Builds an iterator over the positions of the mod-minimizers in the given sequence.
    #[inline]
    pub fn iter_pos(self, seq: &[u8]) -> ModSamplingPosIterator<T, S> {
        assert!(
            self.minimizer_size >= R,
            "mod-minimizers require minimizer_size ≥ r={R}"
        );
        ModSamplingPosIterator::new(
            seq,
            self.minimizer_size,
            self.width,
            R + ((self.minimizer_size - R) % self.width as usize),
            self.hasher,
            self.encoding,
        )
    }
}

impl<T: PrimInt + Hash, S: BuildHasher> MinimizerBuilder<T, ModMinimizer, S, true> {
    /// Builds an iterator over the canonical mod-minimizers and their positions in the given sequence with a boolean indicating a reverse complement.
    /// It requires an odd width to break ties between multiple minimizers.
    #[inline]
    pub fn iter(self, seq: &[u8]) -> CanonicalModSamplingIterator<T, S> {
        assert!(
            self.minimizer_size >= R,
            "mod-minimizers require minimizer_size ≥ r={R}"
        );
        assert_eq!(
            self.width % 2,
            1,
            "width must be odd to break ties between multiple minimizers"
        );
        CanonicalModSamplingIterator::new(
            seq,
            self.minimizer_size,
            self.width,
            R + ((self.minimizer_size - R) % self.width as usize),
            self.hasher,
            self.encoding,
        )
    }

    /// Builds an iterator over the positions of the canonical mod-minimizers in the given sequence with a boolean indicating a reverse complement.
    /// It requires an odd width to break ties between multiple minimizers.
    #[inline]
    pub fn iter_pos(self, seq: &[u8]) -> CanonicalModSamplingPosIterator<T, S> {
        assert!(
            self.minimizer_size >= R,
            "mod-minimizers require minimizer_size ≥ r={R}"
        );
        assert_eq!(
            self.width % 2,
            1,
            "width must be odd to break ties between multiple minimizers"
        );
        CanonicalModSamplingPosIterator::new(
            seq,
            self.minimizer_size,
            self.width,
            R + ((self.minimizer_size - R) % self.width as usize),
            self.hasher,
            self.encoding,
        )
    }
}

impl<T: PrimInt + Hash, A: MinimizerAlgorithm> MinimizerBuilder<T, A, DefaultHashBuilder> {
    fn _new() -> Self {
        let mut encoding = [0u8; 256];
        encoding[b'A' as usize] = 0b00;
        encoding[b'a' as usize] = 0b00;
        encoding[b'C' as usize] = 0b01;
        encoding[b'c' as usize] = 0b01;
        encoding[b'G' as usize] = 0b10;
        encoding[b'g' as usize] = 0b10;
        encoding[b'T' as usize] = 0b11;
        encoding[b't' as usize] = 0b11;
        Self {
            minimizer_size: 21,
            width: 31 - 21 + 1,
            hasher: DefaultHashBuilder::default(),
            encoding,
            _marker: PhantomData,
        }
    }

    /// Sets the seed of the default hasher.
    pub fn seed(mut self, seed: u64) -> Self {
        self.hasher = DefaultHashBuilder::with_seed(seed);
        self
    }
}

impl<T: PrimInt + Hash, A: MinimizerAlgorithm, S: BuildHasher, const CANONICAL: bool>
    MinimizerBuilder<T, A, S, CANONICAL>
{
    /// Sets the size of the minimizers.
    pub fn minimizer_size(mut self, minimizer_size: usize) -> Self {
        let max_size = (T::zero().count_zeros() / 2) as usize;
        assert!(
            minimizer_size <= max_size,
            "With this integer type, minimizer_size must be ≤ {max_size}. Please select a smaller size or a larger type."
        );
        self.minimizer_size = minimizer_size;
        self
    }

    /// Sets the width of the window.
    pub const fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Sets the hasher used to compute minimizers.
    pub fn hasher<H: BuildHasher>(self, hasher: H) -> MinimizerBuilder<T, A, H, CANONICAL> {
        MinimizerBuilder::<T, A, H, CANONICAL> {
            minimizer_size: self.minimizer_size,
            width: self.width,
            hasher,
            encoding: self.encoding,
            _marker: self._marker,
        }
    }

    /// Sets the binary encoding of the bases.
    pub fn encoding(mut self, a: u8, c: u8, g: u8, t: u8) -> Self {
        self.encoding[b'A' as usize] = a;
        self.encoding[b'a' as usize] = a;
        self.encoding[b'C' as usize] = c;
        self.encoding[b'c' as usize] = c;
        self.encoding[b'G' as usize] = g;
        self.encoding[b'g' as usize] = g;
        self.encoding[b'T' as usize] = t;
        self.encoding[b't' as usize] = t;
        self
    }

    /// Compute canonical minimizers.
    pub fn canonical(self) -> MinimizerBuilder<T, A, S, true> {
        MinimizerBuilder::<T, A, S, true> {
            minimizer_size: self.minimizer_size,
            width: self.width,
            hasher: self.hasher,
            encoding: self.encoding,
            _marker: self._marker,
        }
    }

    /// Compute non-canonical minimizers.
    pub fn non_canonical(self) -> MinimizerBuilder<T, A, S, false> {
        MinimizerBuilder::<T, A, S, false> {
            minimizer_size: self.minimizer_size,
            width: self.width,
            hasher: self.hasher,
            encoding: self.encoding,
            _marker: self._marker,
        }
    }
}

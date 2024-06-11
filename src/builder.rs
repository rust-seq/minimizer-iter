use crate::iterator::{MinimizerIterator, MinimizerPosIterator};
use core::hash::{BuildHasher, Hash};
use core::marker::PhantomData;
pub use minimizer_queue::DefaultHashBuilder;
use minimizer_queue::{ImplicitMinimizerQueue, MinimizerQueue};
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
pub struct MinimizerBuilder<T: PrimInt = u64, S: BuildHasher = DefaultHashBuilder> {
    minimizer_size: usize,
    width: u16,
    hasher: S,
    encoding: [u8; 256],
    _marker: PhantomData<T>,
}

impl<T: PrimInt + Hash> MinimizerBuilder<T, DefaultHashBuilder> {
    /// Sets up the `MinimizerBuilder` with default values:
    /// - minimizer_size = 20
    /// - width = 12 (31 - 20 + 1)
    /// - hasher = [`DefaultHashBuilder`]
    /// - encoding: A = `00`, C = `01`, G = `10`, T = `11`
    pub fn new() -> Self {
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
            minimizer_size: 20,
            width: 31 - 20 + 1,
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

impl<T: PrimInt + Hash> Default for MinimizerBuilder<T, DefaultHashBuilder> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PrimInt + Hash, S: BuildHasher> MinimizerBuilder<T, S> {
    /// Sets the size of the minimizers.
    pub fn minimizer_size(mut self, minimizer_size: usize) -> Self {
        let max_size = (T::zero().count_zeros() / 2) as usize;
        assert!(
            minimizer_size <= max_size,
            "Minimizer size must be ≤ {max_size}."
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
    pub fn hasher<H: BuildHasher>(self, hasher: H) -> MinimizerBuilder<T, H> {
        MinimizerBuilder::<T, H> {
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

    /// Builds an iterator over the minimizers and their positions in the given sequence.
    pub fn iter(self, seq: &[u8]) -> MinimizerIterator<T, S> {
        let queue = MinimizerQueue::with_hasher(self.width, self.hasher);
        let width = self.width as usize;
        MinimizerIterator {
            seq,
            queue,
            width,
            mmer: T::zero(),
            mmer_mask: (T::one() << (2 * self.minimizer_size)) - T::one(),
            encoding: self.encoding,
            base_width: width + self.minimizer_size - 1,
            end: width + self.minimizer_size - 1,
            min_pos: (T::zero(), 0),
        }
    }

    /// Builds an iterator over the positions of the minimizers in the given sequence.
    pub fn iter_pos(self, seq: &[u8]) -> MinimizerPosIterator<T, S> {
        let queue = ImplicitMinimizerQueue::with_hasher(self.width, self.hasher);
        let width = self.width as usize;
        MinimizerPosIterator {
            seq,
            queue,
            width,
            mmer: T::zero(),
            mmer_mask: (T::one() << (2 * self.minimizer_size)) - T::one(),
            encoding: self.encoding,
            base_width: width + self.minimizer_size - 1,
            end: width + self.minimizer_size - 1,
            min_pos: 0,
        }
    }
}

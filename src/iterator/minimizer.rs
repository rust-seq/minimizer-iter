use core::cmp::min;
use core::hash::{BuildHasher, Hash};
use minimizer_queue::{DefaultHashBuilder, ImplicitMinimizerQueue, MinimizerQueue};
use num_traits::{AsPrimitive, PrimInt};
use std::collections::VecDeque;

/// An iterator over the positions of the minimizers of a sequence.
pub struct MinimizerPosIterator<'a, T: PrimInt + Hash = u64, S: BuildHasher = DefaultHashBuilder> {
    pub(crate) seq: &'a [u8],
    pub(crate) queue: ImplicitMinimizerQueue<S>,
    pub(crate) width: usize,
    pub(crate) mmer: T,
    pub(crate) mmer_mask: T,
    pub(crate) encoding: [u8; 256],
    pub(crate) base_width: usize,
    pub(crate) min_pos: usize,
    pub(crate) end: usize,
}

impl<'a, T: PrimInt + Hash, S: BuildHasher> MinimizerPosIterator<'a, T, S> {
    pub fn new(
        seq: &'a [u8],
        minimizer_size: usize,
        width: u16,
        hasher: S,
        encoding: [u8; 256],
    ) -> Self {
        let queue = ImplicitMinimizerQueue::with_hasher(width, hasher);
        let width = width as usize;
        Self {
            seq,
            queue,
            width,
            mmer: T::zero(),
            mmer_mask: (T::one() << (2 * minimizer_size)) - T::one(),
            encoding,
            base_width: width + minimizer_size - 1,
            end: width + minimizer_size - 1,
            min_pos: 0,
        }
    }
}

impl<'a, T: PrimInt + Hash + 'static, S: BuildHasher> Iterator for MinimizerPosIterator<'a, T, S>
where
    u8: AsPrimitive<T>,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            if self.base_width > self.seq.len() {
                return None;
            }
            for i in 0..(self.base_width - self.width) {
                self.mmer = (self.mmer << 2)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
            }
            for i in (self.base_width - self.width)..self.base_width {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.queue.insert(&self.mmer);
            }
            self.min_pos = self.queue.get_min_pos();
        } else {
            let mut min_pos = self.min_pos;
            while self.end < self.seq.len() && min_pos == self.min_pos {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[self.end] as usize) }.as_());
                self.queue.insert(&self.mmer);
                self.end += 1;
                min_pos = self.end - self.base_width + self.queue.get_min_pos();
            }
            if min_pos == self.min_pos {
                return None;
            }
            self.min_pos = min_pos;
        }
        Some(self.min_pos)
    }
}

/// An iterator over the minimizers of a sequence and their positions.
pub struct MinimizerIterator<'a, T: PrimInt + Hash = u64, S: BuildHasher = DefaultHashBuilder> {
    pub(crate) seq: &'a [u8],
    pub(crate) queue: MinimizerQueue<T, S>,
    pub(crate) width: usize,
    pub(crate) mmer: T,
    pub(crate) mmer_mask: T,
    pub(crate) encoding: [u8; 256],
    pub(crate) base_width: usize,
    pub(crate) min_pos: (T, usize),
    pub(crate) end: usize,
}

impl<'a, T: PrimInt + Hash, S: BuildHasher> MinimizerIterator<'a, T, S> {
    pub fn new(
        seq: &'a [u8],
        minimizer_size: usize,
        width: u16,
        hasher: S,
        encoding: [u8; 256],
    ) -> Self {
        let queue = MinimizerQueue::with_hasher(width, hasher);
        let width = width as usize;
        Self {
            seq,
            queue,
            width,
            mmer: T::zero(),
            mmer_mask: (T::one() << (2 * minimizer_size)) - T::one(),
            encoding,
            base_width: width + minimizer_size - 1,
            end: width + minimizer_size - 1,
            min_pos: (T::zero(), 0),
        }
    }
}

impl<'a, T: PrimInt + Hash + 'static, S: BuildHasher> Iterator for MinimizerIterator<'a, T, S>
where
    u8: AsPrimitive<T>,
{
    type Item = (T, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            if self.base_width > self.seq.len() {
                return None;
            }
            for i in 0..(self.base_width - self.width) {
                self.mmer = (self.mmer << 2)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
            }
            for i in (self.base_width - self.width)..self.base_width {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.queue.insert(self.mmer);
            }
            self.min_pos = self.queue.get_min_pos();
        } else {
            let mut min_pos = self.min_pos;
            while self.end < self.seq.len() && min_pos.1 == self.min_pos.1 {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[self.end] as usize) }.as_());
                self.queue.insert(self.mmer);
                self.end += 1;
                let _min_pos = self.queue.get_min_pos();
                min_pos = (_min_pos.0, self.end - self.base_width + _min_pos.1);
            }
            if min_pos.1 == self.min_pos.1 {
                return None;
            }
            self.min_pos = min_pos;
        }
        Some(self.min_pos)
    }
}

/// An iterator over the positions of the canonical minimizers of a sequence with a boolean indicating a reverse complement.
/// It requires an odd width to break ties between multiple minimizers.
pub struct CanonicalMinimizerPosIterator<
    'a,
    T: PrimInt + Hash = u64,
    S: BuildHasher = DefaultHashBuilder,
> {
    pub(crate) seq: &'a [u8],
    pub(crate) queue: ImplicitMinimizerQueue<S>,
    pub(crate) width: usize,
    pub(crate) mmer: T,
    pub(crate) rc_mmer: T,
    pub(crate) mmer_mask: T,
    pub(crate) rc_mmer_shift: usize,
    pub(crate) is_rc: VecDeque<bool>,
    pub(crate) encoding: [u8; 256],
    pub(crate) rc_encoding: [u8; 256],
    pub(crate) base_width: usize,
    pub(crate) min_pos: (usize, bool),
    pub(crate) end: usize,
}

impl<'a, T: PrimInt + Hash, S: BuildHasher> CanonicalMinimizerPosIterator<'a, T, S> {
    pub fn new(
        seq: &'a [u8],
        minimizer_size: usize,
        width: u16,
        hasher: S,
        encoding: [u8; 256],
    ) -> Self {
        let queue = ImplicitMinimizerQueue::with_hasher(width, hasher);
        let width = width as usize;
        assert_eq!(
            width % 2,
            1,
            "width must be odd to break ties between multiple minimizers"
        );
        let mut rc_encoding = encoding;
        rc_encoding.swap(b'A' as usize, b'T' as usize);
        rc_encoding.swap(b'a' as usize, b't' as usize);
        rc_encoding.swap(b'C' as usize, b'G' as usize);
        rc_encoding.swap(b'c' as usize, b'g' as usize);
        Self {
            seq,
            queue,
            width,
            mmer: T::zero(),
            rc_mmer: T::zero(),
            mmer_mask: (T::one() << (2 * minimizer_size)) - T::one(),
            rc_mmer_shift: 2 * (minimizer_size - 1),
            is_rc: VecDeque::with_capacity(width),
            encoding,
            rc_encoding,
            base_width: width + minimizer_size - 1,
            end: width + minimizer_size - 1,
            min_pos: (0, false),
        }
    }

    #[inline]
    fn window_not_canonical(&self) -> bool {
        self.is_rc[self.width / 2]
    }
}

impl<'a, T: PrimInt + Hash + 'static, S: BuildHasher> Iterator
    for CanonicalMinimizerPosIterator<'a, T, S>
where
    u8: AsPrimitive<T>,
{
    type Item = (usize, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            if self.base_width > self.seq.len() {
                return None;
            }
            for i in 0..(self.base_width - self.width) {
                self.mmer = (self.mmer << 2)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[i] as usize) }.as_()
                        << self.rc_mmer_shift);
            }
            for i in (self.base_width - self.width)..self.base_width {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[i] as usize) }.as_()
                        << self.rc_mmer_shift);
                let canonical_mmer = min(self.mmer, self.rc_mmer);
                self.queue.insert(&canonical_mmer);
                self.is_rc.push_back(canonical_mmer == self.rc_mmer);
            }
            let pos = if self.queue.multiple_mins() {
                let (pos, tie) = self.queue.get_inner_min_pos();
                tie.map_or(pos, |alt| {
                    if self.window_not_canonical() {
                        alt
                    } else {
                        pos
                    }
                })
            } else {
                self.queue.get_min_pos()
            };
            self.min_pos = (pos, self.is_rc[pos])
        } else {
            let mut min_pos = self.min_pos;
            while self.end < self.seq.len() && min_pos == self.min_pos {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[self.end] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[self.end] as usize) }
                        .as_()
                        << self.rc_mmer_shift);
                let canonical_mmer = min(self.mmer, self.rc_mmer);
                self.queue.insert(&canonical_mmer);
                self.is_rc.pop_front();
                self.is_rc.push_back(canonical_mmer == self.rc_mmer);
                self.end += 1;
                let pos = if self.queue.multiple_mins() {
                    let (pos, tie) = self.queue.get_inner_min_pos();
                    tie.map_or(pos, |alt| {
                        if self.window_not_canonical() {
                            alt
                        } else {
                            pos
                        }
                    })
                } else {
                    self.queue.get_min_pos()
                };
                min_pos = (self.end - self.base_width + pos, self.is_rc[pos]);
            }
            if min_pos == self.min_pos {
                return None;
            }
            self.min_pos = min_pos;
        }
        Some(self.min_pos)
    }
}

/// An iterator over the canonical minimizers of a sequence and their positions with a boolean indicating a reverse complement.
/// It requires an odd width to break ties between multiple minimizers.
pub struct CanonicalMinimizerIterator<
    'a,
    T: PrimInt + Hash = u64,
    S: BuildHasher = DefaultHashBuilder,
> {
    pub(crate) seq: &'a [u8],
    pub(crate) queue: MinimizerQueue<T, S>,
    pub(crate) width: usize,
    pub(crate) mmer: T,
    pub(crate) rc_mmer: T,
    pub(crate) mmer_mask: T,
    pub(crate) rc_mmer_shift: usize,
    pub(crate) is_rc: VecDeque<bool>,
    pub(crate) encoding: [u8; 256],
    pub(crate) rc_encoding: [u8; 256],
    pub(crate) base_width: usize,
    pub(crate) min_pos: (T, usize, bool),
    pub(crate) end: usize,
}

impl<'a, T: PrimInt + Hash, S: BuildHasher> CanonicalMinimizerIterator<'a, T, S> {
    pub fn new(
        seq: &'a [u8],
        minimizer_size: usize,
        width: u16,
        hasher: S,
        encoding: [u8; 256],
    ) -> Self {
        let queue = MinimizerQueue::with_hasher(width, hasher);
        let width = width as usize;
        assert_eq!(
            width % 2,
            1,
            "width must be odd to break ties between multiple minimizers"
        );
        let mut rc_encoding = encoding;
        rc_encoding.swap(b'A' as usize, b'T' as usize);
        rc_encoding.swap(b'a' as usize, b't' as usize);
        rc_encoding.swap(b'C' as usize, b'G' as usize);
        rc_encoding.swap(b'c' as usize, b'g' as usize);
        Self {
            seq,
            queue,
            width,
            mmer: T::zero(),
            rc_mmer: T::zero(),
            mmer_mask: (T::one() << (2 * minimizer_size)) - T::one(),
            rc_mmer_shift: 2 * (minimizer_size - 1),
            is_rc: VecDeque::with_capacity(width),
            encoding,
            rc_encoding,
            base_width: width + minimizer_size - 1,
            end: width + minimizer_size - 1,
            min_pos: (T::zero(), 0, false),
        }
    }

    #[inline]
    fn window_not_canonical(&self) -> bool {
        self.is_rc[self.width / 2]
    }
}

impl<'a, T: PrimInt + Hash + 'static, S: BuildHasher> Iterator
    for CanonicalMinimizerIterator<'a, T, S>
where
    u8: AsPrimitive<T>,
{
    type Item = (T, usize, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            if self.base_width > self.seq.len() {
                return None;
            }
            for i in 0..(self.base_width - self.width) {
                self.mmer = (self.mmer << 2)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[i] as usize) }.as_()
                        << self.rc_mmer_shift);
            }
            for i in (self.base_width - self.width)..self.base_width {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[i] as usize) }.as_()
                        << self.rc_mmer_shift);
                let canonical_mmer = min(self.mmer, self.rc_mmer);
                self.queue.insert(canonical_mmer);
                self.is_rc.push_back(canonical_mmer == self.rc_mmer);
            }
            let _min_pos = if self.queue.multiple_mins() {
                let (x, pos, tie) = self.queue.get_inner_min_pos();
                tie.map_or((x, pos), |alt| {
                    if self.window_not_canonical() {
                        alt
                    } else {
                        (x, pos)
                    }
                })
            } else {
                self.queue.get_min_pos()
            };
            self.min_pos = (_min_pos.0, _min_pos.1, self.is_rc[_min_pos.1]);
        } else {
            let mut min_pos = self.min_pos;
            while self.end < self.seq.len() && min_pos.1 == self.min_pos.1 {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[self.end] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[self.end] as usize) }
                        .as_()
                        << self.rc_mmer_shift);
                let canonical_mmer = min(self.mmer, self.rc_mmer);
                self.queue.insert(canonical_mmer);
                self.is_rc.pop_front();
                self.is_rc.push_back(canonical_mmer == self.rc_mmer);
                self.end += 1;
                let _min_pos = if self.queue.multiple_mins() {
                    let (x, pos, tie) = self.queue.get_inner_min_pos();
                    tie.map_or((x, pos), |alt| {
                        if self.window_not_canonical() {
                            alt
                        } else {
                            (x, pos)
                        }
                    })
                } else {
                    self.queue.get_min_pos()
                };
                min_pos = (
                    _min_pos.0,
                    self.end - self.base_width + _min_pos.1,
                    self.is_rc[_min_pos.1],
                );
            }
            if min_pos.1 == self.min_pos.1 {
                return None;
            }
            self.min_pos = min_pos;
        }
        Some(self.min_pos)
    }
}

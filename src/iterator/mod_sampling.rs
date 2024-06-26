use core::cmp::min;
use core::hash::{BuildHasher, Hash};
use minimizer_queue::{DefaultHashBuilder, ImplicitMinimizerQueue};
use num_traits::{AsPrimitive, PrimInt};
use std::collections::VecDeque;
use strength_reduce::StrengthReducedU16;

/// An iterator over the positions of the mod-sampling minimizers of a sequence.
pub struct ModSamplingPosIterator<'a, T: PrimInt + Hash = u64, S: BuildHasher = DefaultHashBuilder>
{
    pub(crate) seq: &'a [u8],
    pub(crate) queue: ImplicitMinimizerQueue<S>,
    pub(crate) width_m: StrengthReducedU16,
    pub(crate) width_t: usize,
    pub(crate) tmer: T,
    pub(crate) tmer_mask: T,
    pub(crate) encoding: [u8; 256],
    pub(crate) base_width: usize,
    pub(crate) min_pos: usize,
    pub(crate) end: usize,
}

impl<'a, T: PrimInt + Hash, S: BuildHasher> ModSamplingPosIterator<'a, T, S> {
    pub fn new(
        seq: &'a [u8],
        minimizer_size: usize,
        width: u16,
        t: usize,
        hasher: S,
        encoding: [u8; 256],
    ) -> Self {
        let width_m = StrengthReducedU16::new(width);
        let width_t = width + (minimizer_size - t) as u16;
        let queue = ImplicitMinimizerQueue::with_hasher(width_t, hasher);
        let width_t = width_t as usize;
        Self {
            seq,
            queue,
            width_m,
            width_t,
            tmer: T::zero(),
            tmer_mask: (T::one() << (2 * t)) - T::one(),
            encoding,
            base_width: width_t + t - 1,
            end: width_t + t - 1,
            min_pos: 0,
        }
    }
}

impl<'a, T: PrimInt + Hash + 'static, S: BuildHasher> Iterator for ModSamplingPosIterator<'a, T, S>
where
    u8: AsPrimitive<T>,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            if self.base_width > self.seq.len() {
                return None;
            }
            for i in 0..(self.base_width - self.width_t) {
                self.tmer = (self.tmer << 2)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
            }
            for i in (self.base_width - self.width_t)..self.base_width {
                self.tmer = ((self.tmer << 2) & self.tmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.queue.insert(&self.tmer);
            }
            self.min_pos = (self.queue.get_min_pos() as u16 % self.width_m) as usize;
        } else {
            let mut min_pos = self.min_pos;
            while self.end < self.seq.len() && min_pos == self.min_pos {
                self.tmer = ((self.tmer << 2) & self.tmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[self.end] as usize) }.as_());
                self.queue.insert(&self.tmer);
                self.end += 1;
                min_pos = self.end - self.base_width
                    + (self.queue.get_min_pos() as u16 % self.width_m) as usize;
            }
            if min_pos == self.min_pos {
                return None;
            }
            self.min_pos = min_pos;
        }
        Some(self.min_pos)
    }
}

/// An iterator over the mod-sampling minimizers of a sequence and their positions.
pub struct ModSamplingIterator<'a, T: PrimInt + Hash = u64, S: BuildHasher = DefaultHashBuilder> {
    pub(crate) seq: &'a [u8],
    pub(crate) queue: ImplicitMinimizerQueue<S>,
    pub(crate) width_m: StrengthReducedU16,
    pub(crate) width_t: usize,
    pub(crate) mmer: T,
    pub(crate) mmer_mask: T,
    pub(crate) tmer_mask: T,
    pub(crate) canon_mmers: VecDeque<T>,
    pub(crate) encoding: [u8; 256],
    pub(crate) base_width: usize,
    pub(crate) min_pos: (T, usize),
    pub(crate) end: usize,
}

impl<'a, T: PrimInt + Hash, S: BuildHasher> ModSamplingIterator<'a, T, S> {
    pub fn new(
        seq: &'a [u8],
        minimizer_size: usize,
        width: u16,
        t: usize,
        hasher: S,
        encoding: [u8; 256],
    ) -> Self {
        let width_m = StrengthReducedU16::new(width);
        let width_t = width + (minimizer_size - t) as u16;
        let queue = ImplicitMinimizerQueue::with_hasher(width_t, hasher);
        let width_t = width_t as usize;
        Self {
            seq,
            queue,
            width_m,
            width_t,
            mmer: T::zero(),
            mmer_mask: (T::one() << (2 * minimizer_size)) - T::one(),
            tmer_mask: (T::one() << (2 * t)) - T::one(),
            canon_mmers: VecDeque::with_capacity(width as usize),
            encoding,
            base_width: width_t + t - 1,
            end: width_t + t - 1,
            min_pos: (T::zero(), 0),
        }
    }
}

impl<'a, T: PrimInt + Hash + 'static, S: BuildHasher> Iterator for ModSamplingIterator<'a, T, S>
where
    u8: AsPrimitive<T>,
{
    type Item = (T, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            if self.base_width > self.seq.len() {
                return None;
            }
            let width_m = self.width_m.get() as usize;
            for i in 0..(self.base_width - self.width_t) {
                self.mmer = (self.mmer << 2)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
            }
            for i in (self.base_width - self.width_t)..(self.base_width - width_m) {
                self.mmer = (self.mmer << 2)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.queue.insert(&(self.mmer & self.tmer_mask));
            }
            for i in (self.base_width - width_m)..self.base_width {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.queue.insert(&(self.mmer & self.tmer_mask));
                self.canon_mmers.push_back(self.mmer);
            }
            let _min_pos = (self.queue.get_min_pos() as u16 % self.width_m) as usize;
            self.min_pos = (self.canon_mmers[_min_pos], _min_pos);
        } else {
            let mut min_pos = self.min_pos;
            while self.end < self.seq.len() && min_pos.1 == self.min_pos.1 {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[self.end] as usize) }.as_());
                self.queue.insert(&(self.mmer & self.tmer_mask));
                self.canon_mmers.pop_front();
                self.canon_mmers.push_back(self.mmer);
                self.end += 1;
                let _min_pos = (self.queue.get_min_pos() as u16 % self.width_m) as usize;
                min_pos = (
                    self.canon_mmers[_min_pos],
                    self.end - self.base_width + _min_pos,
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

/// An iterator over the positions of the canonical mod-sampling minimizers of a sequence with a boolean indicating a reverse complement.
/// It requires an odd width to break ties between multiple minimizers.
pub struct CanonicalModSamplingPosIterator<
    'a,
    T: PrimInt + Hash = u64,
    S: BuildHasher = DefaultHashBuilder,
> {
    pub(crate) seq: &'a [u8],
    pub(crate) queue: ImplicitMinimizerQueue<S>,
    pub(crate) width_m: StrengthReducedU16,
    pub(crate) width_t: usize,
    pub(crate) mmer: T,
    pub(crate) rc_mmer: T,
    pub(crate) mmer_mask: T,
    pub(crate) tmer_mask: T,
    pub(crate) rc_mmer_shift: usize,
    pub(crate) rc_tmer_shift: usize,
    pub(crate) is_rc_m: VecDeque<bool>,
    pub(crate) encoding: [u8; 256],
    pub(crate) rc_encoding: [u8; 256],
    pub(crate) base_width: usize,
    pub(crate) min_pos: (usize, bool),
    pub(crate) end: usize,
}

impl<'a, T: PrimInt + Hash, S: BuildHasher> CanonicalModSamplingPosIterator<'a, T, S> {
    pub fn new(
        seq: &'a [u8],
        minimizer_size: usize,
        width: u16,
        t: usize,
        hasher: S,
        encoding: [u8; 256],
    ) -> Self {
        let width_m = StrengthReducedU16::new(width);
        let width_t = width + (minimizer_size - t) as u16;
        assert_eq!(
            width_t % width_m,
            0,
            "(minimizer_size - t) must be a multiple of the width to preserve canonical minimizers"
        );
        assert_eq!(
            width % 2,
            1,
            "width must be odd to break ties between multiple minimizers"
        );
        let queue = ImplicitMinimizerQueue::with_hasher(width_t, hasher);
        let width_t = width_t as usize;
        let mut rc_encoding = encoding;
        rc_encoding.swap(b'A' as usize, b'T' as usize);
        rc_encoding.swap(b'a' as usize, b't' as usize);
        rc_encoding.swap(b'C' as usize, b'G' as usize);
        rc_encoding.swap(b'c' as usize, b'g' as usize);
        Self {
            seq,
            queue,
            width_m,
            width_t,
            mmer: T::zero(),
            rc_mmer: T::zero(),
            mmer_mask: (T::one() << (2 * minimizer_size)) - T::one(),
            tmer_mask: (T::one() << (2 * t)) - T::one(),
            rc_mmer_shift: 2 * (minimizer_size - 1),
            rc_tmer_shift: 2 * (minimizer_size - t),
            is_rc_m: VecDeque::with_capacity(width as usize),
            encoding,
            rc_encoding,
            base_width: width_t + t - 1,
            end: width_t + t - 1,
            min_pos: (0, false),
        }
    }

    #[inline]
    fn window_not_canonical(&self) -> bool {
        let mid = self.is_rc_m.len() / 2;
        self.is_rc_m[mid]
    }
}

impl<'a, T: PrimInt + Hash + 'static, S: BuildHasher> Iterator
    for CanonicalModSamplingPosIterator<'a, T, S>
where
    u8: AsPrimitive<T>,
{
    type Item = (usize, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            if self.base_width > self.seq.len() {
                return None;
            }
            let width_m = self.width_m.get() as usize;
            for i in 0..(self.base_width - self.width_t) {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[i] as usize) }.as_()
                        << self.rc_mmer_shift);
            }
            for i in (self.base_width - self.width_t)..(self.base_width - width_m) {
                self.mmer = (self.mmer << 2)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[i] as usize) }.as_()
                        << self.rc_mmer_shift);
                let tmer = self.mmer & self.tmer_mask;
                let rc_tmer = self.rc_mmer >> self.rc_tmer_shift;
                let canonical_tmer = min(tmer, rc_tmer);
                self.queue.insert(&canonical_tmer);
            }
            for i in (self.base_width - width_m)..self.base_width {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[i] as usize) }.as_()
                        << self.rc_mmer_shift);
                let tmer = self.mmer & self.tmer_mask;
                let rc_tmer = self.rc_mmer >> self.rc_tmer_shift;
                let canonical_tmer = min(tmer, rc_tmer);
                self.queue.insert(&canonical_tmer);
                self.is_rc_m.push_back(self.rc_mmer <= self.mmer);
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
            let pos = (pos as u16 % self.width_m) as usize;
            self.min_pos = (pos, self.is_rc_m[pos]);
        } else {
            let mut min_pos = self.min_pos;
            while self.end < self.seq.len() && min_pos.0 == self.min_pos.0 {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[self.end] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[self.end] as usize) }
                        .as_()
                        << self.rc_mmer_shift);
                let tmer = self.mmer & self.tmer_mask;
                let rc_tmer = self.rc_mmer >> self.rc_tmer_shift;
                let canonical_tmer = min(tmer, rc_tmer);
                self.queue.insert(&canonical_tmer);
                self.is_rc_m.pop_front();
                self.is_rc_m.push_back(self.rc_mmer <= self.mmer);
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
                let pos = (pos as u16 % self.width_m) as usize;
                min_pos = (self.end - self.base_width + pos, self.is_rc_m[pos]);
            }
            if min_pos.0 == self.min_pos.0 {
                return None;
            }
            self.min_pos = min_pos;
        }
        Some(self.min_pos)
    }
}

/// An iterator over the canonical mod-sampling minimizers of a sequence and their positions with a boolean indicating a reverse complement.
/// It requires an odd width to break ties between multiple minimizers.
pub struct CanonicalModSamplingIterator<
    'a,
    T: PrimInt + Hash = u64,
    S: BuildHasher = DefaultHashBuilder,
> {
    pub(crate) seq: &'a [u8],
    pub(crate) queue: ImplicitMinimizerQueue<S>,
    pub(crate) width_m: StrengthReducedU16,
    pub(crate) width_t: usize,
    pub(crate) mmer: T,
    pub(crate) rc_mmer: T,
    pub(crate) mmer_mask: T,
    pub(crate) tmer_mask: T,
    pub(crate) rc_mmer_shift: usize,
    pub(crate) rc_tmer_shift: usize,
    pub(crate) canon_mmers: VecDeque<(T, bool)>,
    pub(crate) encoding: [u8; 256],
    pub(crate) rc_encoding: [u8; 256],
    pub(crate) base_width: usize,
    pub(crate) min_pos: (T, usize, bool),
    pub(crate) end: usize,
}

impl<'a, T: PrimInt + Hash, S: BuildHasher> CanonicalModSamplingIterator<'a, T, S> {
    pub fn new(
        seq: &'a [u8],
        minimizer_size: usize,
        width: u16,
        t: usize,
        hasher: S,
        encoding: [u8; 256],
    ) -> Self {
        let width_m = StrengthReducedU16::new(width);
        let width_t = width + (minimizer_size - t) as u16;
        assert_eq!(
            width_t % width_m,
            0,
            "(minimizer_size - t) must be a multiple of the width to preserve canonical minimizers"
        );
        assert_eq!(
            width % 2,
            1,
            "width must be odd to break ties between multiple minimizers"
        );
        let queue = ImplicitMinimizerQueue::with_hasher(width_t, hasher);
        let width_t = width_t as usize;
        let mut rc_encoding = encoding;
        rc_encoding.swap(b'A' as usize, b'T' as usize);
        rc_encoding.swap(b'a' as usize, b't' as usize);
        rc_encoding.swap(b'C' as usize, b'G' as usize);
        rc_encoding.swap(b'c' as usize, b'g' as usize);
        Self {
            seq,
            queue,
            width_m,
            width_t,
            mmer: T::zero(),
            rc_mmer: T::zero(),
            mmer_mask: (T::one() << (2 * minimizer_size)) - T::one(),
            tmer_mask: (T::one() << (2 * t)) - T::one(),
            rc_mmer_shift: 2 * (minimizer_size - 1),
            rc_tmer_shift: 2 * (minimizer_size - t),
            canon_mmers: VecDeque::with_capacity(width as usize),
            encoding,
            rc_encoding,
            base_width: width_t + t - 1,
            end: width_t + t - 1,
            min_pos: (T::zero(), 0, false),
        }
    }

    #[inline]
    fn window_not_canonical(&self) -> bool {
        let mid = self.canon_mmers.len() / 2;
        self.canon_mmers[mid].1
    }
}

impl<'a, T: PrimInt + Hash + 'static, S: BuildHasher> Iterator
    for CanonicalModSamplingIterator<'a, T, S>
where
    u8: AsPrimitive<T>,
{
    type Item = (T, usize, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            if self.base_width > self.seq.len() {
                return None;
            }
            let width_m = self.width_m.get() as usize;
            for i in 0..(self.base_width - self.width_t) {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[i] as usize) }.as_()
                        << self.rc_mmer_shift);
            }
            for i in (self.base_width - self.width_t)..(self.base_width - width_m) {
                self.mmer = (self.mmer << 2)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[i] as usize) }.as_()
                        << self.rc_mmer_shift);
                let tmer = self.mmer & self.tmer_mask;
                let rc_tmer = self.rc_mmer >> self.rc_tmer_shift;
                let canonical_tmer = min(tmer, rc_tmer);
                self.queue.insert(&canonical_tmer);
            }
            for i in (self.base_width - width_m)..self.base_width {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[i] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[i] as usize) }.as_()
                        << self.rc_mmer_shift);
                let tmer = self.mmer & self.tmer_mask;
                let rc_tmer = self.rc_mmer >> self.rc_tmer_shift;
                let canonical_tmer = min(tmer, rc_tmer);
                self.queue.insert(&canonical_tmer);
                let canonical_mmer = min(self.mmer, self.rc_mmer);
                self.canon_mmers
                    .push_back((canonical_mmer, canonical_mmer == self.rc_mmer));
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
            let pos = (pos as u16 % self.width_m) as usize;
            let (mmer, rc) = self.canon_mmers[pos];
            self.min_pos = (mmer, pos, rc);
        } else {
            let mut min_pos = self.min_pos;
            while self.end < self.seq.len() && min_pos.1 == self.min_pos.1 {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { self.encoding.get_unchecked(self.seq[self.end] as usize) }.as_());
                self.rc_mmer = (self.rc_mmer >> 2)
                    | (unsafe { self.rc_encoding.get_unchecked(self.seq[self.end] as usize) }
                        .as_()
                        << self.rc_mmer_shift);
                let tmer = self.mmer & self.tmer_mask;
                let rc_tmer = self.rc_mmer >> self.rc_tmer_shift;
                let canonical_tmer = min(tmer, rc_tmer);
                self.queue.insert(&canonical_tmer);
                let canonical_mmer = min(self.mmer, self.rc_mmer);
                self.canon_mmers.pop_front();
                self.canon_mmers
                    .push_back((canonical_mmer, canonical_mmer == self.rc_mmer));
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
                let pos = (pos as u16 % self.width_m) as usize;
                let (mmer, rc) = self.canon_mmers[pos];
                min_pos = (mmer, self.end - self.base_width + pos, rc);
            }
            if min_pos.1 == self.min_pos.1 {
                return None;
            }
            self.min_pos = min_pos;
        }
        Some(self.min_pos)
    }
}

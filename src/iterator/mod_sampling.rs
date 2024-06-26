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

use core::hash::{BuildHasher, Hash};
pub use minimizer_queue::DefaultHashBuilder;
use minimizer_queue::{ImplicitMinimizerQueue, MinimizerQueue};
use num_traits::{AsPrimitive, PrimInt};

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
                    | (unsafe { *self.encoding.as_ptr().add(self.seq[i] as usize) }.as_());
            }
            for i in (self.base_width - self.width)..self.base_width {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { *self.encoding.as_ptr().add(self.seq[i] as usize) }.as_());
                self.queue.insert(&self.mmer);
            }
            self.min_pos = self.queue.get_min_pos();
            Some(self.min_pos)
        } else {
            let mut min_pos = self.min_pos;
            while self.end < self.seq.len() && min_pos == self.min_pos {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { *self.encoding.as_ptr().add(self.seq[self.end] as usize) }.as_());
                self.queue.insert(&self.mmer);
                self.end += 1;
                min_pos = self.end - self.base_width + self.queue.get_min_pos();
            }
            if self.end >= self.seq.len() {
                return None;
            }
            self.min_pos = min_pos;
            Some(self.min_pos)
        }
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
                    | (unsafe { *self.encoding.as_ptr().add(self.seq[i] as usize) }.as_());
            }
            for i in (self.base_width - self.width)..self.base_width {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { *self.encoding.as_ptr().add(self.seq[i] as usize) }.as_());
                self.queue.insert(self.mmer);
            }
            self.min_pos = self.queue.get_min_pos();
            Some(self.min_pos)
        } else {
            let mut min_pos = self.min_pos;
            while self.end < self.seq.len() && min_pos.1 == self.min_pos.1 {
                self.mmer = ((self.mmer << 2) & self.mmer_mask)
                    | (unsafe { *self.encoding.as_ptr().add(self.seq[self.end] as usize) }.as_());
                self.queue.insert(self.mmer);
                self.end += 1;
                let _min_pos = self.queue.get_min_pos();
                min_pos = (_min_pos.0, self.end - self.base_width + _min_pos.1);
            }
            if self.end >= self.seq.len() {
                return None;
            }
            self.min_pos = min_pos;
            Some(self.min_pos)
        }
    }
}

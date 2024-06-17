pub mod algorithm;
mod builder;
pub mod iterator;

pub use builder::MinimizerBuilder;
pub use minimizer_queue::DefaultHashBuilder;

#[cfg(test)]
mod tests {
    use super::*;
    use nohash_hasher::BuildNoHashHasher;

    #[test]
    fn test_minimizer_iter() {
        let seq = b"TGATTGCACAATC";
        let minimizer_size = 3;
        let width = 4;
        let hasher = BuildNoHashHasher::<u64>::default();
        let mut min_iter = MinimizerBuilder::new()
            .minimizer_size(minimizer_size)
            .width(width)
            .hasher(hasher)
            .iter(seq);

        assert_eq!(min_iter.next(), Some((0b001111, 2))); // ATT
        assert_eq!(min_iter.next(), Some((0b010001, 6))); // CAC
        assert_eq!(min_iter.next(), Some((0b000100, 7))); // ACA
        assert_eq!(min_iter.next(), Some((0b000011, 9))); // AAT
        assert_eq!(min_iter.next(), None);
    }

    #[test]
    fn test_minimizer_iter_pos() {
        let seq = b"TGATTGCACAATC";
        let minimizer_size = 3;
        let width = 4;
        let hasher = BuildNoHashHasher::<u64>::default();
        let mut min_iter = MinimizerBuilder::<u64>::new()
            .minimizer_size(minimizer_size)
            .width(width)
            .hasher(hasher)
            .iter_pos(seq);

        assert_eq!(min_iter.next(), Some(2)); // ATT
        assert_eq!(min_iter.next(), Some(6)); // CAC
        assert_eq!(min_iter.next(), Some(7)); // ACA
        assert_eq!(min_iter.next(), Some(9)); // AAT
        assert_eq!(min_iter.next(), None);
    }

    #[test]
    fn test_mod_minimizer_iter() {
        let seq = b"TGATTGCACAATC";
        let minimizer_size = 4;
        let width = 4;
        let hasher = BuildNoHashHasher::<u64>::default();
        let mut min_iter = MinimizerBuilder::new_mod()
            .minimizer_size(minimizer_size)
            .width(width)
            .hasher(hasher)
            .iter(seq);

        assert_eq!(min_iter.next(), Some((0b00111110, 2))); // ATTG
        assert_eq!(min_iter.next(), Some((0b01000100, 6))); // CACA
        assert_eq!(min_iter.next(), Some((0b00010000, 7))); // ACAA
        assert_eq!(min_iter.next(), Some((0b00001101, 9))); // AATC
        assert_eq!(min_iter.next(), None);
    }

    #[test]
    fn test_mod_minimizer_iter_pos() {
        let seq = b"TGATTGCACAATC";
        let minimizer_size = 4;
        let width = 4;
        let hasher = BuildNoHashHasher::<u64>::default();
        let mut min_iter = MinimizerBuilder::<u64, _>::new_mod()
            .minimizer_size(minimizer_size)
            .width(width)
            .hasher(hasher)
            .iter_pos(seq);

        assert_eq!(min_iter.next(), Some(2)); // ATTG
        assert_eq!(min_iter.next(), Some(6)); // CACA
        assert_eq!(min_iter.next(), Some(7)); // ACAA
        assert_eq!(min_iter.next(), Some(9)); // AATC
        assert_eq!(min_iter.next(), None);
    }
}

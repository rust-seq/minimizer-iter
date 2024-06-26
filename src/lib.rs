pub mod algorithm;
mod builder;
pub mod iterator;

pub use builder::MinimizerBuilder;
pub use minimizer_queue::DefaultHashBuilder;

#[cfg(test)]
mod tests {
    use super::*;
    use biotest::Format;
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

    fn gen_seq(len: usize) -> Vec<u8> {
        let mut rng = biotest::rand();
        let mut seq = Vec::with_capacity(len);
        let generator = biotest::Sequence::builder()
            .sequence_len(len)
            .build()
            .unwrap();
        generator.record(&mut seq, &mut rng).unwrap();
        seq
    }

    fn rc(seq: &[u8]) -> Vec<u8> {
        seq.iter()
            .rev()
            .map(|&b| match b {
                b'A' => b'T',
                b'a' => b't',
                b'T' => b'A',
                b't' => b'a',
                b'C' => b'G',
                b'c' => b'g',
                b'G' => b'C',
                b'g' => b'c',
                b => b,
            })
            .collect()
    }

    #[test]
    fn test_canonical_minimizer_iter() {
        let seq_len = 1_000_000;
        let seq = &gen_seq(seq_len);
        let seq_rc = &rc(seq);
        let minimizer_size = 21;
        let width = 11;

        let mins: Vec<u64> = MinimizerBuilder::new()
            .canonical()
            .minimizer_size(minimizer_size)
            .width(width)
            .iter(seq)
            .map(|(min, _, _)| min)
            .collect();
        let mut mins_rc: Vec<u64> = MinimizerBuilder::new()
            .canonical()
            .minimizer_size(minimizer_size)
            .width(width)
            .iter(seq_rc)
            .map(|(min, _, _)| min)
            .collect();
        mins_rc.reverse();

        assert_eq!(mins, mins_rc);
    }

    #[test]
    fn test_canonical_minimizer_iter_pos() {
        let seq_len = 1_000_000;
        let seq = &gen_seq(seq_len);
        let seq_rc = &rc(seq);
        let minimizer_size = 21;
        let width = 11;

        let mins: Vec<_> = MinimizerBuilder::<u64>::new()
            .canonical()
            .minimizer_size(minimizer_size)
            .width(width)
            .iter_pos(seq)
            .map(|(pos, _)| pos)
            .collect();
        let mut mins_rc: Vec<_> = MinimizerBuilder::<u64>::new()
            .canonical()
            .minimizer_size(minimizer_size)
            .width(width)
            .iter_pos(seq_rc)
            .map(|(pos, _)| seq_len - pos - minimizer_size)
            .collect();
        mins_rc.reverse();

        assert_eq!(mins, mins_rc);
    }

    #[test]
    fn test_canonical_mod_minimizer_iter() {
        let seq_len = 1_000_000;
        let seq = &gen_seq(seq_len);
        let seq_rc = &rc(seq);
        let minimizer_size = 21;
        let width = 11;

        let mins: Vec<_> = MinimizerBuilder::<u64, _>::new_mod()
            .canonical()
            .minimizer_size(minimizer_size)
            .width(width)
            .iter(seq)
            .map(|(min, _, _)| min)
            .collect();
        let mut mins_rc: Vec<_> = MinimizerBuilder::<u64, _>::new_mod()
            .canonical()
            .minimizer_size(minimizer_size)
            .width(width)
            .iter(seq_rc)
            .map(|(min, _, _)| min)
            .collect();
        mins_rc.reverse();

        assert_eq!(mins, mins_rc);
    }

    #[test]
    fn test_canonical_mod_minimizer_iter_pos() {
        let seq_len = 1_000_000;
        let seq = &gen_seq(seq_len);
        let seq_rc = &rc(seq);
        let minimizer_size = 21;
        let width = 11;
        let mins: Vec<_> = MinimizerBuilder::<u64, _>::new_mod()
            .canonical()
            .minimizer_size(minimizer_size)
            .width(width)
            .iter_pos(seq)
            .map(|(pos, _)| pos)
            .collect();
        let mut mins_rc: Vec<_> = MinimizerBuilder::<u64, _>::new_mod()
            .canonical()
            .minimizer_size(minimizer_size)
            .width(width)
            .iter_pos(seq_rc)
            .map(|(pos, _)| seq_len - pos - minimizer_size)
            .collect();
        mins_rc.reverse();

        assert_eq!(mins, mins_rc);
    }

    #[test]
    fn test_repetitive_minimizer_iter_pos() {
        const SEQ_LEN: usize = 100;
        let seq = &[b'A'; SEQ_LEN];
        let seq_rc = &rc(seq);
        let minimizer_size = 21;
        let width = 11;

        let mins: Vec<_> = MinimizerBuilder::<u64>::new()
            .canonical()
            .minimizer_size(minimizer_size)
            .width(width)
            .iter_pos(seq)
            .map(|(pos, _)| pos)
            .collect();
        let mut mins_rc: Vec<_> = MinimizerBuilder::<u64>::new()
            .canonical()
            .minimizer_size(minimizer_size)
            .width(width)
            .iter_pos(seq_rc)
            .map(|(pos, _)| SEQ_LEN - pos - minimizer_size)
            .collect();
        mins_rc.reverse();

        assert_eq!(mins, mins_rc);
    }

    #[test]
    fn test_repetitive_2_minimizer_iter_pos() {
        const SEQ_LEN: usize = 100;
        let seq = &mut [b'A'; SEQ_LEN];
        for i in (1..SEQ_LEN).step_by(2) {
            seq[i] = b'G';
        }
        let seq_rc = &rc(seq);
        let minimizer_size = 21;
        let width = 11;

        let mins: Vec<_> = MinimizerBuilder::<u64>::new()
            .canonical()
            .minimizer_size(minimizer_size)
            .width(width)
            .iter_pos(seq)
            .map(|(pos, _)| pos)
            .collect();
        let mut mins_rc: Vec<_> = MinimizerBuilder::<u64>::new()
            .canonical()
            .minimizer_size(minimizer_size)
            .width(width)
            .iter_pos(seq_rc)
            .map(|(pos, _)| SEQ_LEN - pos - minimizer_size)
            .collect();
        mins_rc.reverse();

        assert_eq!(mins, mins_rc);
    }
}

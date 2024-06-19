use biotest::Format;
use cocktail::tokenizer::minimizer::{method::Random, Forward};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use minimizer_iter::MinimizerBuilder;
use minimizers::{order::RandomOrder, Minimizer, ModSampling, SamplingScheme};
use nohash_hasher::BuildNoHashHasher;

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

fn minimizer(c: &mut Criterion, seq: &[u8], m: usize, w: u16) {
    let id = format!("minimizer m={m} w={w}");
    c.bench_function(id.as_str(), |b| {
        b.iter(|| {
            for x in MinimizerBuilder::<u64>::new()
                .minimizer_size(m)
                .width(w)
                .iter_pos(seq)
            {
                black_box(x);
            }
        })
    });
}

fn mod_minimizer(c: &mut Criterion, seq: &[u8], m: usize, w: u16) {
    let id = format!("mod-minimizer m={m} w={w}");
    c.bench_function(id.as_str(), |b| {
        b.iter(|| {
            for x in MinimizerBuilder::<u64, _>::new_mod()
                .minimizer_size(m)
                .width(w)
                .iter_pos(seq)
            {
                black_box(x);
            }
        })
    });
}

fn lex_minimizer(c: &mut Criterion, seq: &[u8], m: usize, w: u16) {
    let id = format!("lex minimizer m={m} w={w}");
    c.bench_function(id.as_str(), |b| {
        b.iter(|| {
            for x in MinimizerBuilder::<u64>::new()
                .minimizer_size(m)
                .width(w)
                .hasher(BuildNoHashHasher::<u64>::default())
                .iter_pos(seq)
            {
                black_box(x);
            }
        })
    });
}

fn lex_mod_minimizer(c: &mut Criterion, seq: &[u8], m: usize, w: u16) {
    let id = format!("lex mod-minimizer m={m} w={w}");
    c.bench_function(id.as_str(), |b| {
        b.iter(|| {
            for x in MinimizerBuilder::<u64, _>::new_mod()
                .minimizer_size(m)
                .width(w)
                .hasher(BuildNoHashHasher::<u64>::default())
                .iter_pos(seq)
            {
                black_box(x);
            }
        })
    });
}

fn ragnar_minimizer(c: &mut Criterion, seq: &[u8], m: usize, w: usize) {
    let id = format!("ragnar's random minimizer m={m} w={w}");
    c.bench_function(id.as_str(), |b| {
        b.iter(|| {
            for x in Minimizer::new(m, w, RandomOrder).stream(seq) {
                black_box(x);
            }
        })
    });
}

fn ragnar_mod_minimizer(c: &mut Criterion, seq: &[u8], m: usize, w: usize) {
    let id = format!("ragnar's mod-minimizer m={m} w={w}");
    c.bench_function(id.as_str(), |b| {
        b.iter(|| {
            for x in ModSampling::mod_minimizer(m, w).stream(seq) {
                black_box(x);
            }
        })
    });
}

fn cocktail_minimizer_forward(c: &mut Criterion, seq: &[u8], k: u8, m: u8) {
    let id = format!("cocktail minimizer forward m={m} w={}", k - m + 1);
    c.bench_function(id.as_str(), |b| {
        b.iter(|| {
            for x in Forward::<Random, u64>::new(seq, k, m) {
                black_box(x);
            }
        })
    });
}

fn all_benches(c: &mut Criterion) {
    let seq = gen_seq(1_000_000);
    let ks = [31, 63];
    let ms = [21, 31];
    for (k, m) in ks.iter().copied().zip(ms.iter().copied()) {
        let w = (k - m + 1) as u16;
        minimizer(c, &seq, m, w);
        lex_minimizer(c, &seq, m, w);
        ragnar_minimizer(c, &seq, m, w as usize);
        if k <= 32 {
            cocktail_minimizer_forward(c, &seq, k as u8, m as u8);
        }
        mod_minimizer(c, &seq, m, w);
        lex_mod_minimizer(c, &seq, m, w);
        ragnar_mod_minimizer(c, &seq, m, w as usize);
    }
}

criterion_group!(benches, all_benches);
criterion_main!(benches);

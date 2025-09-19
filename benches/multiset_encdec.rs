
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use dealrs::combrs::binom;
use dealrs::combrs::multiset_decode;
use dealrs::rng_from_seed;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

fn bench_dec<R: Rng>(rng: &mut R, n: usize, k: usize, choose_nk: usize, seq: &mut [usize]) {
    let idx = rng.random_range(0..choose_nk);
    multiset_decode(idx, n, k, seq);
    // assert!(seq[0] <= seq[1]);
}

fn from_elem(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiset_encdec");
    let mut rng = SmallRng::from_rng(&mut rng_from_seed(Some("seed1234")));

    for n in 1..=20 {
        for k in 1..=n {
            let mut seq = [0usize; 64];
            let choose_nk = binom(n + k - 1, k);
            group.bench_with_input(BenchmarkId::from_parameter(format!("n={},k={}", n, k)), &(n, k, choose_nk), |b, &(n, k, choose_nk)| {
                b.iter(|| bench_dec(&mut rng, n, k, choose_nk, &mut seq));
            });
        }
    }
    group.finish();
}

criterion_group!(benches, from_elem);
criterion_main!(benches);

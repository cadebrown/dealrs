
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use dealrs::combrs::binom;
use dealrs::combrs::bagspace::BagSpace;
use dealrs::rng_from_seed;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

fn bench_dec<R: Rng>(rng: &mut R, bagspace: &BagSpace, choose_nk: usize, seq: &mut [usize]) {
    let idx = rng.random_range(0..choose_nk);
    bagspace.dec(idx, seq);
    // assert!(seq[0] <= seq[1]);
}

fn from_elem(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiset_encdec");
    let mut rng = SmallRng::from_rng(&mut rng_from_seed(Some("seed1234")));

    for n in 1..=20 {
        for k in 1..=n {
            let mut seq = [0usize; 64];
            let choose_nk = binom(n + k - 1, k);
            let bagspace = BagSpace::new(n, k);
            group.bench_with_input(BenchmarkId::from_parameter(format!("n={},k={}", n, k)), &(n, k, bagspace), |b, &(n, k, bagspace)| {
                b.iter(|| bench_dec(&mut rng, &bagspace, choose_nk, &mut seq[..k]));
            });
        }
    }
    group.finish();
}

criterion_group!(benches, from_elem);
criterion_main!(benches);

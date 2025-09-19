use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use dealrs::deck::sample_cards;
use dealrs::deck::CardMask;
use dealrs::rng_from_seed;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

fn do_sample_cards<R: Rng>(num: usize, rng: &mut R) {
    sample_cards(CardMask::full(), num, rng);
}

fn from_elem(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample_cards");
    let mut rng = SmallRng::from_rng(&mut rng_from_seed(Some("test")));
    for num in (1..=52).rev() {
        group.bench_with_input(BenchmarkId::from_parameter(num), &num, |b, &num| {
            b.iter(|| do_sample_cards(num, &mut rng));
            });
        }
    group.finish();
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use dealrs::deck::sample_cards;
use dealrs::deck::CardMask;
use dealrs::hand::lutbest5::LutBest5;
use dealrs::hand::Rank5;
use dealrs::rng_from_seed;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

fn do_anyrank5<R: Rng, E: Rank5>(rng: &mut R, engine: &E, num: usize) {
    let cards = sample_cards(CardMask::full(), num, rng);
    let rank = engine.rank5(cards);
    assert!(rank > 0);
}

fn bench_anyrank5<E: Rank5>(c: &mut Criterion, name: &str, engine: &E) {
    let mut group = c.benchmark_group(name);
    let mut rng = SmallRng::from_rng(&mut rng_from_seed(Some("seed1234")));
    for num in 5..=15 {
        group.bench_with_input(BenchmarkId::from_parameter(num), &num, |b, &num| {
            b.iter(|| do_anyrank5(&mut rng, engine, num));
        });
    }
    group.finish();
}

fn from_elem(c: &mut Criterion) {
    bench_anyrank5(c, "lutbest5", &LutBest5::new());
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
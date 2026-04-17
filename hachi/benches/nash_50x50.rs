use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use rand_distr::Uniform;
use hachi::solver::nash_equilibrium;

fn random_payoff_matrix(n: usize, rng: &mut impl Rng) -> Vec<Vec<f64>> {
    let dist = Uniform::new(-1.0, 1.0);
    (0..n)
        .map(|_| (0..n).map(|_| rng.sample(dist)).collect())
        .collect()
}

fn bench_nash_50(c: &mut Criterion) {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);

    let mut group = c.benchmark_group("nash_equilibrium_50x50");
    group.sample_size(50);           // number of samples for statistics
    group.measurement_time(std::time::Duration::from_secs(30));

    // Warm-up + one representative run
    let payoff = random_payoff_matrix(50, &mut rng);
    let (row, col, val) = nash_equilibrium(&payoff); // call your function
    println!("Sample run → value = {:.6}, row sum = {:.4}, col sum = {:.4}",
             val, row.iter().sum::<f64>(), col.iter().sum::<f64>());

    group.bench_function("random_uniform_-1_1", |b| {
        b.iter_batched(
            || random_payoff_matrix(50, &mut rng),
            |payoff| black_box(nash_equilibrium(&payoff)),
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, bench_nash_50);
criterion_main!(benches);
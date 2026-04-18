use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use rand_distr::Uniform;
use hachi::solver::{nash_equilibrium, nash_equilibrium_exact};

fn random_payoff_matrix(n: usize, rng: &mut impl Rng) -> Vec<Vec<f64>> {
    let dist = Uniform::new(-1.0, 1.0);
    (0..n)
        .map(|_| (0..n).map(|_| rng.sample(dist)).collect())
        .collect()
}

// helpers

/// Bench a single (size, solver) combination inside a named group.
macro_rules! bench_size {
    ($c:expr, $n:expr, $label:expr, $solver:expr) => {{
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let group_name = format!("nash_{}x{}_{}", $n, $n, $label);
        let mut group = $c.benchmark_group(&group_name);

        group.sample_size(100);
        group.measurement_time(std::time::Duration::from_secs(10));

        let warmup = random_payoff_matrix($n, &mut rng);
        let (row, col, val) = $solver(&warmup);
        println!(
            "[{}] warm-up → value = {:.6}, row_sum = {:.4}, col_sum = {:.4}",
            group_name,
            val,
            row.iter().sum::<f64>(),
            col.iter().sum::<f64>()
        );

        group.bench_function("random_uniform_-1_1", |b| {
            b.iter_batched(
                || random_payoff_matrix($n, &mut rng),
                |payoff| black_box($solver(&payoff)),
                criterion::BatchSize::SmallInput,
            )
        });

        group.finish();
    }};
}

// approximate solver (FP)

fn bench_approx_4x4(c: &mut Criterion) {
    bench_size!(c, 4, "approx", nash_equilibrium);
}

fn bench_approx_8x8(c: &mut Criterion) {
    bench_size!(c, 8, "approx", nash_equilibrium);
}

fn bench_approx_16x16(c: &mut Criterion) {
    bench_size!(c, 16, "approx", nash_equilibrium);
}

fn bench_approx_32x32(c: &mut Criterion) {
    bench_size!(c, 32, "approx", nash_equilibrium);
}

fn bench_approx_64x64(c: &mut Criterion) {
    bench_size!(c, 64, "approx", nash_equilibrium);
}

// exact solver (LP)

fn bench_exact_4x4(c: &mut Criterion) {
    bench_size!(c, 4, "exact", nash_equilibrium_exact);
}

fn bench_exact_8x8(c: &mut Criterion) {
    bench_size!(c, 8, "exact", nash_equilibrium_exact);
}

fn bench_exact_16x16(c: &mut Criterion) {
    bench_size!(c, 16, "exact", nash_equilibrium_exact);
}

fn bench_exact_32x32(c: &mut Criterion) {
    bench_size!(c, 32, "exact", nash_equilibrium_exact);
}

fn bench_exact_64x64(c: &mut Criterion) {
    bench_size!(c, 64, "exact", nash_equilibrium_exact);
}
// registration

criterion_group!(
    benches,
    // approximate
    bench_approx_4x4,
    bench_approx_8x8,
    bench_approx_16x16,
    bench_approx_32x32,
    bench_approx_64x64,
    // exact
    bench_exact_4x4,
    bench_exact_8x8,
    bench_exact_16x16,
    bench_exact_32x32,
    bench_exact_64x64,
);
criterion_main!(benches);
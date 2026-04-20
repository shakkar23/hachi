use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use features::feature_extractor::Features;
use rand::Rng;

use hachi::eval::{eval, eval_batched, ModelType};

fn generate_random_features(rng: &mut impl Rng) -> Features {
    Features::default()
}

fn bench_eval_single(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let f1 = generate_random_features(&mut rng);
    let f2 = generate_random_features(&mut rng);
    
    c.bench_function("eval_single", |b| {
        b.iter(|| {
            eval(
                black_box(&f1),
                black_box(&f2),
                black_box(ModelType::LightGBM_Large),
            )
        })
    });
}

fn bench_eval_batched(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    
    let batch_sizes = [1, 10, 100, 1000, 10000];
    
    let mut group = c.benchmark_group("eval_batched");
    
    for &size in &batch_sizes {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let pairs: Vec<(Features, Features)> = (0..size)
                .map(|_| {
                    (
                        generate_random_features(&mut rng),
                        generate_random_features(&mut rng),
                    )
                })
                .collect();
            
            let ref_pairs: Vec<(&Features, &Features)> = pairs
                .iter()
                .map(|(f1, f2)| (f1, f2))
                .collect();
            
            b.iter(|| {
                eval_batched(
                    black_box(&ref_pairs),
                    black_box(ModelType::LightGBM_Large),
                )
            })
        });
    }
    
    group.finish();
}

fn bench_feature_conversion(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    
    c.bench_function("feature_to_vec_conversion", |b| {
        let pairs: Vec<(Features, Features)> = (0..100)
            .map(|_| {
                (
                    generate_random_features(&mut rng),
                    generate_random_features(&mut rng),
                )
            })
            .collect();
        
        b.iter(|| {
            let num_rows = pairs.len();
            let mut data = Vec::with_capacity(num_rows * Features::COUNT * 2);
            
            for (f1, f2) in &pairs {
                for v in f1.values() {
                    data.push(black_box(v as f64));
                }
                for v in f2.values() {
                    data.push(black_box(v as f64));
                }
            }
            
            black_box(data)
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(std::time::Duration::from_secs(30))
        .warm_up_time(std::time::Duration::from_secs(5));
    targets = bench_eval_single, bench_eval_batched, bench_feature_conversion
}

criterion_main!(benches);
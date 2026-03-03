use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ndarray::Array2;
use ort::{
    session::{Session, builder::GraphOptimizationLevel},
    value::TensorRef,
};
use std::time::Duration;

fn build_session() -> Session {
    Session::builder()
        .unwrap()
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .unwrap()
        .commit_from_file("models/big_model.onnx")
        .unwrap()
}

fn bench_sequential_calls(c: &mut Criterion) {
    let mut session = build_session();
    let input_data = Array2::from_shape_vec((1, 1650), vec![0.0f32; 1650]).unwrap();

    let mut group = c.benchmark_group("sequential_model_calls");
    group.measurement_time(Duration::from_secs(10));
    group.warm_up_time(Duration::from_secs(3));

    // Single call baseline
    group.throughput(Throughput::Elements(1));
    group.bench_function("single_inference", |b| {
        b.iter(|| {
            let input_ref = TensorRef::from_array_view((
                input_data.shape(),
                input_data.as_slice().unwrap(),
            ))
            .unwrap();
            let outputs = session.run(ort::inputs!["input" => input_ref]).unwrap();
            black_box(outputs["variable"].try_extract_tensor::<f32>().unwrap());
        })
    });

    // Sequential N calls — measures amortized cost and any session state overhead
    for n in [10u64, 100, 1000] {
        group.throughput(Throughput::Elements(n));
        group.bench_with_input(BenchmarkId::new("sequential_n_calls", n), &n, |b, &n| {
            b.iter(|| {
                for _ in 0..n {
                    let input_ref = TensorRef::from_array_view((
                        input_data.shape(),
                        input_data.as_slice().unwrap(),
                    ))
                    .unwrap();
                    let outputs = session.run(ort::inputs!["input" => input_ref]).unwrap();
                    black_box(outputs["variable"].try_extract_tensor::<f32>().unwrap());
                }
            })
        });
    }

    group.finish();
}

fn bench_session_reuse_vs_recreate(c: &mut Criterion) {
    let input_data = Array2::from_shape_vec((1, 1650), vec![0.0f32; 1650]).unwrap();

    let mut group = c.benchmark_group("session_lifecycle");
    group.measurement_time(Duration::from_secs(15));

    // Reuse session across calls
    group.bench_function("reuse_session", |b| {
        let mut session = build_session();
        b.iter(|| {
            let input_ref = TensorRef::from_array_view((
                input_data.shape(),
                input_data.as_slice().unwrap(),
            ))
            .unwrap();
            black_box(session.run(ort::inputs!["input" => input_ref]).unwrap());
        })
    });

    // Recreate session each call (worst case)
    group.bench_function("recreate_session_per_call", |b| {
        b.iter(|| {
            let mut session = build_session();
            let input_ref = TensorRef::from_array_view((
                input_data.shape(),
                input_data.as_slice().unwrap(),
            ))
            .unwrap();
            black_box(session.run(ort::inputs!["input" => input_ref]).unwrap());
        })
    });

    group.finish();
}

criterion_group!(benches, bench_sequential_calls, bench_session_reuse_vs_recreate);
criterion_main!(benches);
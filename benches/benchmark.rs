use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use hive::Hive;

fn criterion_benchmark(c: &mut Criterion) {
    let mut hive = Hive::new();

    c.bench_function("first_benchmark", |b| {
        b.iter(|| {
            let _ = hive.insert(black_box(42));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

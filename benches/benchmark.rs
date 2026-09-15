use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::hint::black_box;

use hive::Hive;

fn insert_benchmark(c: &mut Criterion) {
    let make_hive = || Hive::<i32>::new();
    let insert_16 = |mut hive: Hive<i32>| {
                            for i in 0..16 {
                                let _ = hive.insert(black_box(42));
                            }
                        };

    c.bench_function("insert", |b| {
        b.iter_batched(make_hive, insert_16, BatchSize::SmallInput);
    });
}

fn get_benchmark(c: &mut Criterion) {
    let mut hive: Hive<i32> = Hive::new();
    let handle = hive.insert(42);

    c.bench_function("get", |b| {
        b.iter(black_box(|| hive.get(black_box(handle))));
    });
}

fn iter_benchmark(c: &mut Criterion) {
    let mut handles = Vec::new();
    let mut hive: Hive<i32> = Hive::new();
    for i in 0..16 {
        handles.push(hive.insert(i));
    }

    // randomly remove half
    // TODO not random right now; later select random handles to delete
    for (idx, handle) in handles.iter().enumerate() {
        if idx % 2 == 0 {
            hive.remove(*handle);
        }
    }

    // TODO this is suspciously fast; maybe the compiler is optimizing away things?
    // try to insert random data and delete random handles
    c.bench_function("iter", |b| {
        b.iter(|| black_box( for _ in hive.iter(){}));
    });

}

criterion_group!(benches, iter_benchmark);
criterion_main!(benches);

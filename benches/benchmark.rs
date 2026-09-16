use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::hint::black_box;

use random::*;

use hive::Hive;

// TODO this benchmark is testing insertion of 16 items, not individual insertions. Fix!
fn insert_benchmark(c: &mut Criterion) {
    let make_hive = || Hive::<u64>::new();
    let insert = |mut hive: Hive<u64>| {
                 //           for i in 0..16 {
                                let _ = hive.insert(black_box(42));
                 //           }
                        };

    c.bench_function("insert", |b| {
        b.iter_batched(make_hive, insert, BatchSize::NumIterations(15));
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
    let mut hive: Hive<u64> = Hive::new();

    let mut source = random::default(42);
    for i in 0..16 {
        handles.push(hive.insert(source.read_u64()));
    }

    // TODO this is suspciously fast; maybe the compiler is optimizing away things?
    // try to insert random data and delete random handles
    c.bench_function("iter", |b| {
        b.iter(|| black_box( for _ in hive.iter(){}));
    });

}

fn iter_with_holes_benchmark(c: &mut Criterion) {
    let mut handles = Vec::new();
    let mut hive: Hive<u64> = Hive::new();

    let mut source = random::default(42);

    for i in 0..16 {
        handles.push(hive.insert(source.read_u64()));
    }

    // randomly remove half
    // repeatedly pick random index and swap with the end to shuffle
    // then take half the container and pass that to test closure that calls remove()
    for _ in 0..handles.len()*2 {
        let idx_1 = source.read_u64() as usize % handles.len();
        let idx_2  = source.read_u64() as usize % handles.len(); 
        handles.swap(idx_1, idx_2);
    }

    handles.truncate(handles.len() / 2);
    for i in 0.. (handles.len() / 2) {
        hive.remove(handles[i]);
    }

    // TODO this is suspciously fast; maybe the compiler is optimizing away things?
    // try to insert random data and delete random handles
    c.bench_function("iter", |b| {
        b.iter(|| black_box( for _ in hive.iter(){}));
    });
}

criterion_group!(benches, iter_benchmark);
criterion_main!(benches);

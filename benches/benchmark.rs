use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::hint::black_box;

use random::*;

use hive::Hive;

#[link(name = "AMDuProfileController")]
unsafe extern "C"  {
    #[link_name = "amdProfilePause"]
    fn amd_profile_pause() -> i32;

    #[link_name = "amdProfileResume"]
    fn amd_profile_resume() -> i32;
}

fn profile_pause() {
    unsafe { amd_profile_pause(); }
}

fn profile_resume() {
    unsafe {amd_profile_resume(); }
}

// fill the first block only (avoid allocations)
fn insert_benchmark(c: &mut Criterion) {
    let make_hive = || Hive::<u64>::new();
    let insert = |mut hive: Hive<u64>| {
                                let _ = hive.insert(black_box(42));
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

    c.bench_function("iter", |b| {
        b.iter(|| black_box( hive.iter().count()));
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
        let idx_2  = handles.len()-1; 
        handles.swap(idx_1, idx_2);
    }

    handles.truncate(handles.len() / 2);
    for i in 0.. (handles.len() / 2) {
        hive.remove(handles[i]);
    }

    c.bench_function("iter", |b| {
        b.iter(|| black_box( hive.iter().count()));
    });
}

criterion_group!(benches, insert_benchmark);
criterion_main!(benches);

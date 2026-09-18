use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use core::time;
use std::time::Instant;
use std::hint::black_box;

use random::*;

use hive::Hive;

#[link(name = "AMDProfileController", kind = "static")]
unsafe extern "C"  {
    // #[link_name = "amdProfilePause"]
    fn amdProfilePauseImpl() -> bool;

    // #[link_name = "amdProfileResume"]
    fn amdProfileResumeImpl() -> bool;
}

fn profile_pause() {
    unsafe { amdProfilePauseImpl(); }
}

fn profile_resume() {
    unsafe { amdProfileResumeImpl(); }
}

fn insert_benchmark_uprof(c: &mut Criterion) {
    let make_hive = || Hive::<u64>::new();
    let insert = |mut hive: Hive<u64>| {
                                profile_resume();
                                let _ = hive.insert(black_box(42));
                                profile_pause();
                        };

    c.bench_function("insert", |b| {
        let mut hive = Hive::new();
        let handle = hive.insert(1); // allocate
        hive.remove(handle);                 // doesn't deallocate when empty
        b.iter_custom(move |iters| {
            let insertion_chunks = iters / (hive.capacity() as u64);
            let remainder = iters % (hive.capacity() as u64);
            let capacity = hive.capacity();

            let time = Instant::now();

            for _ in 0..insertion_chunks {
                profile_resume();
                for _ in 0..capacity {
                    hive.insert(42);
                }
                profile_pause();
                hive.clear();
            }

            hive.clear();
            profile_resume();
            for _ in 0..remainder {
                hive.insert(42);
            }
            profile_pause();

            Instant::now() - time
        })
    });
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

criterion_group!(benches, insert_benchmark_uprof);
criterion_main!(benches);

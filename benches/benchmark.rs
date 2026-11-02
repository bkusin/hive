use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::hint::black_box;

use random::*;

use hive::Hive;

#[cfg(feature = "profile_uprof")]
#[link(name = "AMDProfileController", kind = "static")]
unsafe extern "C"  {
    fn amdProfilePauseImpl() -> bool;

    fn amdProfileResumeImpl() -> bool;
}
#[cfg(feature = "profile_uprof")]
pub struct AMDuProf;

#[cfg(feature = "profile_uprof")]
impl Profiler for AmduProf {
    fn start_profiling(&mut self, benchmark_id: &str, benchmark_dir: &Path) {
        unsafe { amdProfileResumeImpl(); }
    }

    fn stop_profiling(&mut self, benchmark_id: &str, benchmark_dir: &Path) {
        unsafe { amdProfilePauseImpl(); }
    }
}

fn configure_criterion() -> Criterion {
    let criterion = Criterion::default();

    #[cfg(feature = "profile_uprof")] {
        criterion.with_profiler(AMDuProf)
    }
    #[cfg(not(feature = "profile_uprof"))] {
        criterion
    }
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
    for _ in 0..16 {
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

    for _ in 0..16 {
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

criterion_group!(name = benches; 
                config = configure_criterion();
                targets = insert_benchmark);
criterion_main!(benches);

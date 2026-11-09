use criterion::{BatchSize::{self, SmallInput}, Criterion, criterion_group, criterion_main, profiler::Profiler};
use std::hint::black_box;

use std::path::Path;

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
impl Profiler for AMDuProf {
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
const DEFAULT_CAPACITY: usize = 256;

// fill the first block only (avoid allocations)
fn insert_benchmark(c: &mut Criterion) {

    // make a hive with one full block less one slot (so insertion doesn't trigger a new block allocation)
    let make_hive = || { 
        let mut hive = Hive::<u64>::with_block_capacity(DEFAULT_CAPACITY);
        
        let handle = hive.insert(0); 
        for _ in 1..DEFAULT_CAPACITY {
            hive.insert(0);
        }

        hive.remove(handle);
        hive
    };
    
    
    let insert = |mut hive: Hive<u64>| {
                                black_box(hive.insert(black_box(42)));
                        };

    c.bench_function("insert", |b| {
        b.iter_batched(make_hive, insert, BatchSize::SmallInput);
    });
}

fn get_benchmark(c: &mut Criterion) {
    let mut hive: Hive<u64> = Hive::with_block_capacity(DEFAULT_CAPACITY);
    let handle = hive.insert(42);

    c.bench_function("get", |b| {
        b.iter(black_box(|| hive.get(black_box(handle))));
    });
}

fn remove_benchmark(c: &mut Criterion) {
    let make_hive= || { 
        let mut hive = Hive::<u64>::with_block_capacity(DEFAULT_CAPACITY); 
        let handle = hive.insert(42);
        (hive, handle)
    };

    c.bench_function("remove", |b| {
        b.iter_batched(
            make_hive,
            |(mut hive, handle)| black_box(hive.remove(black_box(handle))),
            BatchSize::SmallInput,
        );
    });
}

fn iter_benchmark(c: &mut Criterion) {
    let mut handles = Vec::new();
    let mut hive: Hive<u64> = Hive::with_block_capacity(DEFAULT_CAPACITY);

    let mut source = random::default(42);
    for _ in 0..DEFAULT_CAPACITY {
        handles.push(hive.insert(source.read_u64()));
    }

    c.bench_function("iter", |b| {
        b.iter(|| black_box( hive.iter().count()));
    });
}

fn iter_with_holes_benchmark(c: &mut Criterion) {
    let mut handles = Vec::new();
    let mut hive: Hive<u64> = Hive::with_block_capacity(DEFAULT_CAPACITY);

    let mut source = random::default(42);

    for _ in 0..DEFAULT_CAPACITY {
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

    c.bench_function("iter_with_holes", |b| {
        b.iter(|| black_box( hive.iter().count()));
    });
}

criterion_group!(name = benches; 
                config = configure_criterion();
                targets = iter_benchmark);
criterion_main!(benches);

use std::{mem::MaybeUninit, ops::{Index, IndexMut}};

const DEFAULT_CAP: usize = 16;

struct Vacancy {
    block_idx: usize,
    block_offset: usize,
}

struct Hive<T> {
    block_size: usize,
    data: Vec<Box<[MaybeUninit<T>; DEFAULT_CAP]>>,
    liveness: Vec<bool>,
    vacancies: Vec<Vacancy>,
    len: usize,
    capacity: usize,
}

impl<T> Hive<T> {
    
    fn new() -> Self {
        Self {
            block_size: DEFAULT_CAP,
            data: vec![Box::<[MaybeUninit<T>; DEFAULT_CAP]>::new( [const { MaybeUninit::uninit() }; DEFAULT_CAP] )],
            liveness: vec![true; DEFAULT_CAP],
            vacancies: Vec::<Vacancy>::with_capacity(DEFAULT_CAP),
            len: 0,
            capacity: DEFAULT_CAP,
        }
    }

    fn push(&mut self, value: T) {
        let mut block_idx = self.len / self.block_size;
        let mut block_offset = self.len % self.block_size;

        // append, fill vacancy, or expand
        if self.len == self.capacity {
            if !self.vacancies.is_empty() {
                let vacancy = self.vacancies.last().unwrap();
                block_idx = vacancy.block_idx;
                block_offset = vacancy.block_offset;
            }
            else {
                // expand
                self.data.push(Box::<[MaybeUninit<T>; DEFAULT_CAP]>::new([const { MaybeUninit::uninit() }; DEFAULT_CAP]));
                self.capacity += DEFAULT_CAP;
                self.liveness.extend_from_slice(&[false; DEFAULT_CAP]);
                block_idx += 1;
                block_offset = 0;
            }
        }

        (self.data[block_idx])[block_offset].write(value);
        self.liveness[self.len] = true;
        self.len += 1;
    }

    fn delete(&self) {
        todo!();
    }

    fn len(&self) -> usize {
        self.len
    }

    fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<T> Index<usize> for Hive<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        if index >= self.capacity { panic!(); }

        let block_idx = index / self.block_size;
        let idx = index - (block_idx * self.block_size);

        unsafe { (self.data[block_idx])[idx].assume_init_ref() }
    }
}

impl<T> IndexMut<usize> for Hive<T> {
   
    fn index_mut(&mut self, index: usize) -> &mut T {
        if index >= self.capacity { panic!(); }

        let block_idx = index / self.block_size;
        let idx = index - (block_idx * self.block_size);

        unsafe { (self.data[block_idx])[idx].assume_init_mut() }
    }
}

// TODO:
/*
indexing math needs to "skip over" soft-deleted elements
similarly, since we don't actually delete anything, we need to handle dangling references because Rust's lifetime and borrowing rules
won't help with soft-deletes.

This means we need a "virtual length" as if the structure is compacted to close "deletes"
How does C++26 std::hive do it?
The existence of holes at arbitrary locations means the only way to get the correct live index is to scan linearly.
So std::hive does not have indexing. But iterators are valid until the item is deleted or the container is sorted/cleared/resized.

implement Iterator

Benchmark latency stability
*/


mod test {
    use super::*;

    #[test]
    fn test_new() {
        let hive: Hive<i32> = Hive::new();
    }

    #[test]
    fn test_push() {
        let mut hive: Hive<i32> = Hive::new();
        hive.push(42);

        assert_eq!(hive[0], 42);
    }

    #[test]
    fn test_expand() {
        let mut hive: Hive<i32> = Hive::new();
        for i in 0..18 {
            hive.push(i as i32);
        }

        // should access second block
        assert_eq!(hive.capacity(), 32);
        assert_eq!(hive.len(), 18);
        assert_eq!(hive[17], 17);
        assert_eq!(&hive.liveness[0..18], &[true; 18]);
    }

    #[test]
    fn fill_vacancy() {
        todo!()
    }

}
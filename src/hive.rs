use std::{mem::MaybeUninit, ops::{Index, IndexMut}};

const DEFAULT_CAP: usize = 16;

struct Hive<T> {
    block_size: usize,
    data: Vec<Box<[MaybeUninit<T>; DEFAULT_CAP]>>,
    liveness: Vec<bool>,
    len: usize,
    capacity: usize,
}

impl<T> Hive<T> {
    
    fn new() -> Self {
        Self {
            block_size: DEFAULT_CAP,
            data: vec![Box::<[MaybeUninit<T>; DEFAULT_CAP]>::new( [const { MaybeUninit::uninit() }; DEFAULT_CAP] )],
            liveness: vec![true; DEFAULT_CAP],
            len: 0,
            capacity: DEFAULT_CAP,
        }
    }

    fn push(&mut self, value: T) {
        let block_idx = self.len / self.block_size;
        let block_offset = self.len % self.block_size;

        if (self.len == self.capacity) {
            self.data.push(Box::<[MaybeUninit<T>; DEFAULT_CAP]>::new([const { MaybeUninit::uninit() }; DEFAULT_CAP]));
            self.capacity += DEFAULT_CAP;
            self.liveness.extend_from_slice(&[false; DEFAULT_CAP]);
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

}
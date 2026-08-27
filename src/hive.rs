use std::{
    mem::{MaybeUninit, uninitialized}, ops::{Index, IndexMut},
};

const DEFAULT_CAP: usize = 16;

struct Handle {
    block_idx: usize,
    block_offset: usize,
    // TODO: Generations
}

struct Block<T> {
    data: [MaybeUninit<T>; DEFAULT_CAP],
    len: usize,
    liveness: [bool; DEFAULT_CAP],
    free_list: Vec<usize>,         // stack of deleted slots
}

impl<T> Block<T> {
    fn new() -> Self {
        Self {
            data: [ const { MaybeUninit::uninit() }; DEFAULT_CAP],
            len: 0,
            liveness: [false; DEFAULT_CAP],
            free_list: Vec::new(),
        }
    }

    fn insert(&mut self, value: T) -> usize {
        // reuse an empty slot first
        let mut index = 
        if !self.free_list.is_empty() {
            self.free_list.pop().unwrap()
        }
        else {
            // either the block is completely empty or completely full
            // the caller already checks the latter case
            0
        };
            
        self.data[index].write(value);
        self.len += 1;
        index
    }
}

struct Hive<T> {
  //  block_size: usize,
    data: Vec<Box<Block<T>>>,
    blocks_with_vacancies: usize,
}

impl<T> Hive<T> {
    pub fn new() -> Self {
        Self {
           data: Vec::new(),
           blocks_with_vacancies: 0,
        }
    }

    pub fn insert(&mut self, value: T) -> Handle {
        // if there are no blocks with vacancies, create a new block
        if self.blocks_with_vacancies == 0 {
            self.data.push(Box::new(Block::<T>::new()));
            self.blocks_with_vacancies = 1;
        }
    
        // look for a block to insert into
        for block in self.data.iter_mut().enumerate() {
            if block.1.len < DEFAULT_CAP {
                let idx = block.1.insert(value);
                // did the insertion max out a block?
                if block.1.len == DEFAULT_CAP { self.blocks_with_vacancies -= 1; }
                
                return Handle{ block_idx: block.0, block_offset: idx };
            }
        }

        unreachable!("Hive should always have a block with space");
    }

    pub fn get(&self, handle: Handle) -> &T {
        // TODO deal with invalid handle, generation
        unsafe {
            self.data[handle.block_idx].data[handle.block_offset].assume_init_ref()
        }
    }

    pub fn len(&self) -> usize {
        let mut len = 0;
        for block in &self.data {
            len += block.len;
        }
        len
    }

    pub fn capacity(&self) -> usize {
        self.data.len() * DEFAULT_CAP
    }

}


mod test {
    use super::*;

    #[test]
    fn test_new() {
        let hive: Hive<i32> = Hive::new();
    }

    #[test]
    fn test_insert() {
        let mut hive: Hive<i32> = Hive::new();
        let handle = hive.insert(42);

        assert_eq!(*hive.get(handle), 42);
    }

    #[test]
    #[ignore = "To do"]
    fn test_delete() {
        let mut hive: Hive<i32> = Hive::new();
        let handle = hive.insert(42);
        assert_eq!(hive.len(), 1);

    //    hive.delete(handle);
    //    assert_eq!(hive.len(), 0);
    }

    #[test]
    fn test_expand() {
        let mut hive: Hive<i32> = Hive::new();
        for i in 0..18 {
            hive.insert(i as i32);
        }

        // should access second block
        assert_eq!(hive.capacity(), 32);
        assert_eq!(hive.len(), 18);
     //   assert_eq!(hive[17], 17);
     //   assert_eq!(&hive.liveness[0..18], &[true; 18]);
    }

    #[test]
    #[ignore]
    fn fill_vacancy() {
        todo!()
    }
}

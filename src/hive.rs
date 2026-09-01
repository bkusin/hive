use std::{
    mem::{MaybeUninit, uninitialized}, ops::{Index, IndexMut},
};

const DEFAULT_CAP: usize = 16;

#[derive(Copy, Clone)]
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
            // the caller (hive) already checks the latter case
            0
        };
            
        self.data[index].write(value);
        self.liveness[index] = true;
        self.len += 1;
        
        index
    }

    fn remove(&mut self, index: usize) -> Option<T> {
        if self.liveness[index] == false {
            return None
        }

        self.liveness[index] = false;
        self.free_list.push(index);
        self.len -= 1;

        // SAFETY: We already checked the liveness flag
        // FIXME need to use assume_init_read() then assume_init_drop
        unsafe {
            Some(self.data[index].assume_init_read())
        }
    }

    fn get(&self, index: usize) -> Option<&T> {
        if self.liveness[index] == false {
            return None
        }

        // SAFETY: we already checked the liveness
        unsafe {
            Some(self.data[index].assume_init_ref())
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if self.liveness[index] == false {
            return None
        }

        // SAFETY: we already checked the liveness
        unsafe {
            Some(self.data[index].assume_init_mut())
        }
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

    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        // SAFETY: hive doesn't delete or move blocks so block exists
        self.data[handle.block_idx].remove(handle.block_offset)
    }

    pub fn get(&self, handle: Handle) -> Option<&T> {
        // TODO deal with invalid handle, generation -> None
        unsafe {
            self.data[handle.block_idx].get(handle.block_offset)
        }
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        // TODO deal with invalid handle, generation
        unsafe {
            self.data[handle.block_idx].get_mut(handle.block_offset)
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

        let result = *hive.get(handle).unwrap();

        assert_eq!(result, 42);
    }
    
    #[test]
    fn test_mutate() {
        let mut hive: Hive<i32> = Hive::new();
        let handle = hive.insert(42);
        let mutable = hive.get_mut(handle);
        *mutable.unwrap() = 33;

        assert_eq!(*hive.get(handle).unwrap(), 33);
    }

    #[test]
    fn test_remove() {
        let mut hive: Hive<i32> = Hive::new();
        let handle = hive.insert(42);
        hive.remove(handle);

        assert_eq!(hive.get(handle), None);
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

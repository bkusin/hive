use std::{
    mem::{MaybeUninit}
};

const DEFAULT_CAP: usize = 16;

#[derive(Copy, Clone)]
pub struct Handle {
    block_idx: usize,
    block_offset: usize,
    // TODO: Generations
}

// auxiliary data is in its own structures for now
// embedding it into slots could save memory or be more cache efficient
// the liveness flags can be replaced with a bitfield or Matt Bentley's branchless method
struct Block<T> {
    data: Box<[MaybeUninit<T>]>,
    len: usize,
    cap: usize,
    liveness: Box<[bool]>, 
    free_list: Vec<usize>,         // stack of deleted slots
}

impl<T> Block<T> {
    // for fixed-size blocks of user-defined size, new() doesn't make much sense.
    // Neither an empty container nor a default size make sense here.
    #[cfg(any)]
    fn new() -> Self {

        let mut vacancies = Vec::new();

        Self {
            data: Box::new([MaybeUninit::<T>::uninit(); 0]),
            len: 0,
            cap: DEFAULT_CAP,
            liveness: Box::new([false; 0]),
            free_list: vacancies,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        let vacancies = (0..capacity).rev().collect();
        let liveness= Box::<[bool]>::new_zeroed_slice(capacity);
        
        Self {
            data: Box::<[T]>::new_uninit_slice(capacity),
            len: 0,
            cap: capacity,
            // SAFETY zero as construced above is valid for bool so the conversion is safe
            liveness: unsafe { liveness.assume_init() },
            free_list: vacancies,
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
        // we don't need to drop anything because this transfers ownership out of the Block
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

pub struct Hive<T> {
    block_capacity: usize,
    data: Vec<Box<Block<T>>>,  // TODO don't need Box
    blocks_with_vacancies: usize,
}

impl<T> Hive<T> {
    // for fixed-size blocks of user-defined size, new() doesn't make much sense.
    // Neither an empty container nor a default size make sense here.
    #[cfg(any)]
    pub fn new() -> Self {
        Self {
           block_size: 0, 
           data: Vec::new(),
           blocks_with_vacancies: 0,
        }
    }

    pub fn with_block_capacity(capacity: usize) -> Self {
        Self {
            block_capacity: capacity,
            data:Vec::new(),
            blocks_with_vacancies: 0,
        }
    }

    pub fn insert(&mut self, value: T) -> Handle {
        // if there are no blocks with vacancies, create a new block
        if self.blocks_with_vacancies == 0 {
            self.data.push(Box::new(Block::<T>::with_capacity(self.block_capacity)));
            self.blocks_with_vacancies = 1;
        }
    
        // look for a block to insert into
        for block in self.data.iter_mut().enumerate() {
            if block.1.len < self.block_capacity {
                let idx = block.1.insert(value);
                // did the insertion max out a block?
                if block.1.len == self.block_capacity { self.blocks_with_vacancies -= 1; }
                
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
        // SAFETY: hive doesn't delete or move blocks so block exists
        unsafe {
            self.data[handle.block_idx].get(handle.block_offset)
        }
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        // TODO deal with invalid handle, generation
        // SAFETY: hive doesn't delete or move blocks so block exists
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
        self.data.len() * self.block_capacity
    }

    pub fn drain() {
        todo!()
    }

    pub fn clear(&mut self) {
        self.retain(|x| { false });
    }

    // drops the backing store 
    // TODO: Need to consider reuse (e.g., Box::take) or reallocation, fixed or variable sized blocks, set at runtime vs compile time
    pub fn shrink() {
        todo!()
    }

    // FIXME update blocks_with_vacancies
    pub fn retain<F>(&mut self, mut f: F)
        where F: FnMut(&T) -> bool {
            for block in &mut self.data {
                for (index, item) in block.data.iter_mut().enumerate() {
                    unsafe {
                        if block.liveness[index] && !f(item.assume_init_ref()) {
                            block.liveness[index] = false;
                            block.len -= 1;
                            block.free_list.push(index);
                            item.assume_init_drop();
                        }
                    }
                }
            }
    }

    // TODO: more Iterators - &mut and value (move) 
    pub fn iter<'a>(&'a self) -> HiveIterator<'a, T> {
        HiveIterator {
            hive: self,
            current: Handle { block_idx: 0, block_offset: 0 },
         //   end: Handle { block_idx: self.data.len()-1, block_offset: self.data[len()-1].len()-1 },
        }
    } 
}

impl<T> Drop for Hive<T> {
    fn drop(&mut self) {

        for block in self.data.iter_mut().enumerate() {
            for i in 0..block.1.cap {
                if block.1.liveness[i] == true {
                    // SAFETY we're only dropping live things, which are always owned by the container
                    unsafe {
                        block.1.data[i].assume_init_drop();
                    }
                }
            }
        }
    }
}

pub struct HiveIterator<'a, T> {
    hive: &'a Hive<T>,
    current: Handle,
  //  end: Handle,
}

impl<'a, T> Iterator for HiveIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
       while self.current.block_idx < self.hive.data.len() && self.current.block_offset < self.hive.data[self.current.block_idx].cap {
            
            let item = self.hive.get(self.current);

            self.current.block_offset += 1;  
                
            // did we run off the block?
            if self.current.block_offset == self.hive.data[self.current.block_idx].cap {
                self.current.block_idx += 1;
                self.current.block_offset = 0;
            }

            if item.is_none() { continue; }  // skip hole
            else { return item }
       }
       None
    }
}



mod test {
    use super::*;
    const DEFAULT_CAPACITY: usize = 256;

    #[test]
    fn create_with_capacity() {
        let mut hive: Hive<i32> = Hive::with_block_capacity(DEFAULT_CAPACITY);
        assert_eq!(hive.capacity(), 0);

        hive.insert(42);
        assert_eq!(hive.capacity(), DEFAULT_CAPACITY);
    }

    #[test]
    fn insert() {
        let mut hive: Hive<i32> = Hive::with_block_capacity(DEFAULT_CAPACITY);
        let handle = hive.insert(42);

        let result = *hive.get(handle).unwrap();

        assert_eq!(result, 42);
    }
    
    #[test]
    fn mutate() {
        let mut hive: Hive<i32> = Hive::with_block_capacity(DEFAULT_CAPACITY);
        let handle = hive.insert(42);
        let mutable = hive.get_mut(handle);
        *mutable.unwrap() = 33;

        assert_eq!(*hive.get(handle).unwrap(), 33);
    }

    #[test]
    fn remove() {
        let mut hive: Hive<i32> = Hive::with_block_capacity(DEFAULT_CAPACITY);
        let handle = hive.insert(42);
        hive.remove(handle);

        assert_eq!(hive.get(handle), None);
    }

    #[test]
    fn remove_and_refill() {
        let mut hive: Hive<i32> = Hive::with_block_capacity(DEFAULT_CAPACITY);
        let handle = hive.insert(42);
        hive.remove(handle);

        // insertion after removing a single item should fill that same slot
        hive.insert(33);

        assert_eq!(*hive.get(handle).unwrap(), 33);
    } 

    #[test]
    fn expand_hive() {
        let mut hive: Hive<i32> = Hive::with_block_capacity(16);;
        for i in 0..18 {
            hive.insert(i as i32);
        }

        // should have added a second block
        assert_eq!(hive.capacity(), 32);
        assert_eq!(hive.len(), 18);
    }

    #[test]
    fn iterate() {
        let mut hive: Hive<i32> = Hive::with_block_capacity(DEFAULT_CAPACITY);
        for i in 1..=3 {
            hive.insert(i as i32);
        }

        for i in hive.iter() {
            print!("{} ", i);
        }

        let v:Vec<i32>   = vec![1, 2, 3];
        let v2: Vec<i32> = hive.iter().copied().collect();
        assert_eq!(v, v2);
    }

    #[test]
    fn iterate_with_holes() {
        let mut hive: Hive<i32> = Hive::with_block_capacity(DEFAULT_CAPACITY);;
        hive.insert(1);
        let handle = hive.insert(2);
        hive.insert(3);
        hive.insert(4);

        hive.remove(handle);

        for i in hive.iter() {
            print!("{} ", i);   // should be 1,3,4
        }

        let v: Vec<i32>   = vec![1, 3, 4];
        let v2: Vec<i32> = hive.iter().copied().collect();
        assert_eq!(v, v2);
    }

    #[test]
    fn iterate_across_blocks() {
        let mut hive: Hive<i32> = Hive::with_block_capacity(16);;
        for value in 0..18 {
            hive.insert(value);
        }

        let values: Vec<i32> = hive.iter().copied().collect();
        assert_eq!(values, (0..18).collect::<Vec<_>>());
    }
    
    #[test]
    fn drop() {
        let mut hive: Hive<String> = Hive::with_block_capacity(DEFAULT_CAPACITY);;
        let s = String::from("Hello");
        
        hive.insert(s);  // move string
        
        // TODO: verify that memory was freed, e.g. use MIRI
        std::mem::drop(hive);
    }

    #[test]
    fn retain() {
        let mut hive: Hive<i32> = Hive::with_block_capacity(DEFAULT_CAPACITY);;
        for i in 0..10 {
            hive.insert(i);
        }

        // retain even values
        hive.retain(|x| { x % 2 == 0 });

        assert_eq!(hive.len(), 5);
        for item in hive.iter() {
            assert_eq!(item % 2, 0);
        }
    }

    #[test]
    fn clear() {
        let mut hive: Hive<i32> = Hive::with_block_capacity(DEFAULT_CAPACITY);;
        for i in 0..10 {
            hive.insert(i);
        }

        hive.clear();

        assert_eq!(hive.len(), 0);
    }
}
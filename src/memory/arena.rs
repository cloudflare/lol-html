use safemem::copy_over;
use std::mem::size_of;

use super::{MemoryLimitExceededError, SharedMemoryLimiter};

#[derive(Debug)]
pub struct Arena {
    limiter: SharedMemoryLimiter,
    mem_pool: Vec<u8>,
    watermark: usize,
}

impl Arena {
    pub fn new(limiter: SharedMemoryLimiter, preallocated_size: usize) -> Self {
        limiter
            .borrow_mut()
            .preallocate(preallocated_size * size_of::<u8>());

        Arena {
            limiter,
            mem_pool: vec![0; preallocated_size],
            watermark: 0,
        }
    }

    pub fn append(&mut self, slice: &[u8]) -> Result<(), MemoryLimitExceededError> {
        // NOTE: the capacity (i.e. the amount of memory that can be used without reallocation)
        // is basically a current length of the underlying memory pool.
        let capacity = self.mem_pool.len();
        let new_watermark = self.watermark + slice.len();

        if new_watermark > capacity {
            // NOTE: we can't fit in the whole slice with the memory available.
            // Split the slice into two parts: one that we can fit in now and the one
            // for which we need to allocate additional memory.
            let space_left = capacity - self.watermark;
            let (within_capacity, rest) = slice.split_at(space_left);

            // NOTE: ask the limiter if we can have more space
            self.limiter
                .borrow_mut()
                .increase_usage(rest.len() * size_of::<u8>())?;

            self.mem_pool[self.watermark..capacity].copy_from_slice(within_capacity);
            self.mem_pool.extend_from_slice(rest);
        } else {
            self.mem_pool[self.watermark..new_watermark].copy_from_slice(slice);
        }

        self.watermark = new_watermark;

        Ok(())
    }

    pub fn init_with(&mut self, slice: &[u8]) -> Result<(), MemoryLimitExceededError> {
        self.watermark = 0;

        self.append(slice)
    }

    pub fn shrink_to_last(&mut self, byte_count: usize) {
        copy_over(
            &mut self.mem_pool,
            self.watermark - byte_count,
            0,
            byte_count,
        );

        self.watermark = byte_count;
    }

    pub fn bytes(&self) -> &[u8] {
        &self.mem_pool[..self.watermark]
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        self.limiter
            .borrow_mut()
            .decrease_usage(self.mem_pool.len() * size_of::<u8>());
    }
}

#[cfg(test)]
mod tests {
    use super::super::limiter::MemoryLimiter;
    use super::*;
    use std::rc::Rc;

    #[test]
    fn append() {
        let limiter = MemoryLimiter::new_shared(10);
        let mut arena = Arena::new(Rc::clone(&limiter), 2);

        arena.append(&[1, 2]).unwrap();
        assert_eq!(arena.bytes(), &[1, 2]);
        assert_eq!(limiter.borrow().current_usage(), 2);

        arena.append(&[3, 4]).unwrap();
        assert_eq!(arena.bytes(), &[1, 2, 3, 4]);
        assert_eq!(limiter.borrow().current_usage(), 4);

        arena.append(&[5, 6, 7, 8, 9, 10]).unwrap();
        assert_eq!(arena.bytes(), &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(limiter.borrow().current_usage(), 10);

        let err = arena.append(&[11]).unwrap_err();

        assert_eq!(
            err,
            MemoryLimitExceededError {
                current_usage: 11,
                max: 10
            }
        );
    }

    #[test]
    fn init_with() {
        let limiter = MemoryLimiter::new_shared(5);
        let mut arena = Arena::new(Rc::clone(&limiter), 0);

        arena.init_with(&[1]).unwrap();
        assert_eq!(arena.bytes(), &[1]);
        assert_eq!(limiter.borrow().current_usage(), 1);

        arena.append(&[1, 2]).unwrap();
        assert_eq!(arena.bytes(), &[1, 1, 2]);
        assert_eq!(limiter.borrow().current_usage(), 3);

        arena.init_with(&[1, 2, 3]).unwrap();
        assert_eq!(arena.bytes(), &[1, 2, 3]);
        assert_eq!(limiter.borrow().current_usage(), 3);

        arena.init_with(&[]).unwrap();
        assert_eq!(arena.bytes(), &[]);
        assert_eq!(limiter.borrow().current_usage(), 3);

        let err = arena.init_with(&[1, 2, 3, 4, 5, 6, 7]).unwrap_err();

        assert_eq!(
            err,
            MemoryLimitExceededError {
                current_usage: 7,
                max: 5
            }
        );
    }

    #[test]
    fn shrink_to_last() {
        let limiter = MemoryLimiter::new_shared(10);
        let mut arena = Arena::new(Rc::clone(&limiter), 0);

        arena.append(&[0, 1, 2, 3]).unwrap();
        arena.shrink_to_last(2);
        assert_eq!(arena.bytes(), &[2, 3]);
        assert_eq!(limiter.borrow().current_usage(), 4);

        arena.append(&[0, 1]).unwrap();
        assert_eq!(arena.bytes(), &[2, 3, 0, 1]);
        assert_eq!(limiter.borrow().current_usage(), 4);

        arena.shrink_to_last(1);
        assert_eq!(arena.bytes(), &[1]);
        assert_eq!(limiter.borrow().current_usage(), 4);

        arena.append(&[2, 3, 4, 5]).unwrap();
        arena.shrink_to_last(4);
        assert_eq!(arena.bytes(), &[2, 3, 4, 5]);
        assert_eq!(limiter.borrow().current_usage(), 5);
    }
}

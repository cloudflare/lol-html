use safemem::copy_over;
use std::cmp::max;
use std::mem::size_of;

use super::{ExceededLimitsError, SharedMemoryLimiter};

pub struct Buffer {
    limiter: SharedMemoryLimiter,
    data: Vec<u8>,
    length: usize,
    space_left: usize,
}

impl Buffer {
    pub fn try_new(
        limiter: SharedMemoryLimiter,
        initial_size: usize,
    ) -> Result<Buffer, ExceededLimitsError> {
        limiter
            .borrow_mut()
            .increase_mem(initial_size * size_of::<u8>())?;

        Ok(Buffer {
            limiter,
            data: vec![0; initial_size],
            space_left: initial_size,
            length: 0,
        })
    }

    pub fn append(&mut self, slice: &[u8]) -> Result<(), ExceededLimitsError> {
        // check if we can rely on our preallocated capacity
        if self.space_left > 0 {
            let new_space_left = self.space_left as isize - slice.len() as isize;

            // negative space left, we need to notify the limiter
            if new_space_left < 0 {
                self.limiter
                    .borrow_mut()
                    .increase_mem(new_space_left.abs() as usize * size_of::<u8>())?;
            }

            self.space_left = max(new_space_left, 0) as usize;
        } else {
            // the size of the buffer exceeded its initial capacity, we need to notify
            // the ManagedLimiter for future allocations.
            self.limiter
                .borrow_mut()
                .increase_mem(slice.len() * size_of::<u8>())?;
        }

        let new_length = self.length + slice.len();

        self.data.resize(new_length, 0);

        self.data
            .splice(self.length..new_length, slice.iter().cloned());
        self.length = new_length;

        Ok(())
    }

    pub fn init_with(&mut self, slice: &[u8]) -> Result<(), ExceededLimitsError> {
        self.length = 0;
        self.append(slice)
    }

    pub fn shrink_to_last(&mut self, byte_count: usize) {
        copy_over(&mut self.data, self.length - byte_count, 0, byte_count);

        self.length = byte_count;
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data[..self.length]
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.limiter
            .borrow_mut()
            .decrease_mem(self.data.len() * size_of::<u8>());
    }
}

#[cfg(test)]
mod tests {
    use super::super::limiter::MemoryLimiter;
    use super::*;

    #[test]
    fn current_usage() {
        let limiter = MemoryLimiter::new_shared(10);
        let mut buffer = Buffer::try_new(limiter.clone(), 0).unwrap();
        buffer.append(&[0, 0]).unwrap();

        assert_eq!(limiter.borrow().current_usage(), 2);
    }

    #[test]
    fn max_limit() {
        let limiter = MemoryLimiter::new_shared(2);
        let mut buffer = Buffer::try_new(limiter.clone(), 0).unwrap();
        let alloc_err = buffer.append(&[0, 0, 0]).unwrap_err();

        assert_eq!(alloc_err, ExceededLimitsError { current_usage: 3 });
    }

    #[test]
    fn force_allocation_after_initial_capacity() {
        let limiter = MemoryLimiter::new_shared(10);
        let mut buffer = Buffer::try_new(limiter, 2).unwrap();

        buffer.append(&[0, 0]).unwrap(); // pre-allocated
        buffer.append(&[1, 1]).unwrap(); // force allocation

        assert_eq!(buffer.bytes(), &[0, 0, 1, 1]);
    }

    #[test]
    fn shrink_to_last() {
        let limiter = MemoryLimiter::new_shared(10);
        let mut buffer = Buffer::try_new(limiter.clone(), 0).unwrap();

        buffer.append(&[0, 1, 2, 3]).unwrap();
        buffer.shrink_to_last(2);
        assert_eq!(buffer.bytes(), &[2, 3]);

        buffer.append(&[0, 1]).unwrap();
        assert_eq!(buffer.bytes(), &[2, 3, 0, 1]);

        assert_eq!(limiter.borrow().current_usage(), 6);
    }

    #[test]
    fn init_with() {
        let limiter = MemoryLimiter::new_shared(10);
        let mut buffer = Buffer::try_new(limiter.clone(), 0).unwrap();

        buffer.init_with(&[1]).unwrap();
        assert_eq!(limiter.borrow().current_usage(), 1);

        buffer.append(&[1, 2]).unwrap();
        assert_eq!(limiter.borrow().current_usage(), 3);

        buffer.init_with(&[1, 2, 3]).unwrap();
        assert_eq!(limiter.borrow().current_usage(), 6);

        buffer.init_with(&[]).unwrap();
        assert_eq!(limiter.borrow().current_usage(), 6);
    }

    #[test]
    fn preallocated_capacity() {
        let limiter = MemoryLimiter::new_shared(2);
        let mut buffer = Buffer::try_new(limiter.clone(), 2).unwrap();

        buffer.append(&[0]).unwrap();
        buffer.append(&[1]).unwrap();
    }
}

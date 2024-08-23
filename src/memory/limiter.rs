use std::cell::Cell;
use std::rc::Rc;
use thiserror::Error;

/// An error that occures when rewriter exceedes the memory limit specified in the
/// [`MemorySettings`].
///
/// [`MemorySettings`]: ../struct.MemorySettings.html
#[derive(Error, Debug, Eq, PartialEq, Copy, Clone)]
#[error("The memory limit has been exceeded.")]
pub struct MemoryLimitExceededError;

#[derive(Debug, Clone)]
pub struct SharedMemoryLimiter {
    current_usage: Rc<Cell<usize>>,
    max: usize,
}

impl SharedMemoryLimiter {
    pub fn new(max: usize) -> SharedMemoryLimiter {
        SharedMemoryLimiter {
            current_usage: Rc::new(Cell::new(0)),
            max,
        }
    }

    #[cfg(test)]
    pub fn current_usage(&self) -> usize {
        self.current_usage.get()
    }

    #[inline]
    pub fn increase_usage(&self, byte_count: usize) -> Result<(), MemoryLimitExceededError> {
        let previous_usage = self.current_usage.get();
        let current_usage = previous_usage + byte_count;

        self.current_usage.set(current_usage);

        if current_usage > self.max {
            Err(MemoryLimitExceededError)
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn preallocate(&self, byte_count: usize) {
        self.increase_usage(byte_count).expect(
            "Total preallocated memory size should be less than `MemorySettings::max_allowed_memory_usage`.",
        );
    }

    #[inline]
    pub fn decrease_usage(&self, byte_count: usize) {
        let previous_usage = self.current_usage.get();
        let current_usage = previous_usage - byte_count;

        self.current_usage.set(current_usage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_usage() {
        let limiter = SharedMemoryLimiter::new(10);

        assert_eq!(limiter.current_usage(), 0);

        limiter.increase_usage(3).unwrap();
        assert_eq!(limiter.current_usage(), 3);

        limiter.increase_usage(5).unwrap();
        assert_eq!(limiter.current_usage(), 8);

        limiter.decrease_usage(4);
        assert_eq!(limiter.current_usage(), 4);

        let err = limiter.increase_usage(15).unwrap_err();

        assert_eq!(err, MemoryLimitExceededError);
    }

    #[test]
    #[should_panic(
        expected = "Total preallocated memory size should be less than `MemorySettings::max_allowed_memory_usage`."
    )]
    fn preallocate() {
        let limiter = SharedMemoryLimiter::new(10);

        limiter.preallocate(8);
        assert_eq!(limiter.current_usage(), 8);

        limiter.preallocate(10);
    }
}

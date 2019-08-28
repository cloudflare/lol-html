use std::cell::RefCell;
use std::rc::Rc;

use super::ExceededLimitsError;

pub type SharedMemoryLimiter = Rc<RefCell<MemoryLimiter>>;

#[derive(Debug)]
pub struct MemoryLimiter {
    current: usize,
    max: usize,
}
impl MemoryLimiter {
    pub fn new_shared(max: usize) -> SharedMemoryLimiter {
        Rc::new(RefCell::new(MemoryLimiter { max, current: 0 }))
    }

    #[inline]
    #[cfg(feature = "integration_test")]
    pub fn current_usage(&self) -> usize {
        self.current
    }

    #[inline]
    pub fn increase_mem(&mut self, value: usize) -> Result<(), ExceededLimitsError> {
        let new_current = self.current + value;

        if new_current > self.max {
            Err(ExceededLimitsError::new(new_current))
        } else {
            self.current = new_current;
            Ok(())
        }
    }

    #[inline]
    pub fn decrease_mem(&mut self, value: usize) {
        self.current -= value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_capture_usage() {
        let limiter = MemoryLimiter::new_shared(10);
        assert_eq!(limiter.borrow().current_usage(), 0);
    }
}

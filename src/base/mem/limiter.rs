use std::cell::RefCell;
use std::rc::Rc;

pub type SharedMemoryLimiter = Rc<RefCell<MemoryLimiter>>;

#[derive(Fail, Debug, PartialEq)]
#[fail(
    display = "Memory limit of {} bytes has been exceeded: {} bytes were used.",
    max, current_usage
)]
pub struct MemoryLimitExceededError {
    pub current_usage: usize,
    pub max: usize,
}

#[derive(Debug)]
pub struct MemoryLimiter {
    current_usage: usize,
    max: usize,
}

impl MemoryLimiter {
    pub fn new_shared(max: usize) -> SharedMemoryLimiter {
        Rc::new(RefCell::new(MemoryLimiter {
            max,
            current_usage: 0,
        }))
    }

    #[cfg(test)]
    pub fn current_usage(&self) -> usize {
        self.current_usage
    }

    #[inline]
    pub fn increase_usage(&mut self, byte_count: usize) -> Result<(), MemoryLimitExceededError> {
        self.current_usage += byte_count;

        if self.current_usage > self.max {
            Err(MemoryLimitExceededError {
                current_usage: self.current_usage,
                max: self.max,
            })
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn decrease_usage(&mut self, byte_count: usize) {
        self.current_usage -= byte_count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_usage() {
        let limiter = MemoryLimiter::new_shared(10);
        let mut limiter = limiter.borrow_mut();

        assert_eq!(limiter.current_usage(), 0);

        limiter.increase_usage(3).unwrap();
        assert_eq!(limiter.current_usage(), 3);

        limiter.increase_usage(5).unwrap();
        assert_eq!(limiter.current_usage(), 8);

        limiter.decrease_usage(4);
        assert_eq!(limiter.current_usage(), 4);

        let err = limiter.increase_usage(15).unwrap_err();

        assert_eq!(
            err,
            MemoryLimitExceededError {
                current_usage: 19,
                max: 10
            }
        );
    }
}

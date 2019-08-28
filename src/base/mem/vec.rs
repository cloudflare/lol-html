#![allow(clippy::len_without_is_empty)]

use std::mem::size_of;
use std::ops::{Bound, Deref, Index, RangeBounds};
use std::vec::Drain;

use super::{ExceededLimitsError, SharedMemoryLimiter};

#[derive(Debug)]
pub struct LimitedVec<T> {
    limiter: SharedMemoryLimiter,
    vec: Vec<T>,
}
impl<T> LimitedVec<T> {
    pub fn new(limiter: SharedMemoryLimiter) -> Self {
        LimitedVec {
            vec: vec![],
            limiter,
        }
    }

    pub fn push(&mut self, element: T) -> Result<(), ExceededLimitsError> {
        self.limiter.borrow_mut().increase_mem(size_of::<T>())?;
        self.vec.push(element);
        Ok(())
    }

    /// Returns the number of elements in the vector, also referred to as its 'length'.
    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns the last element of the slice, or None if it is empty.
    #[inline]
    pub fn last(&self) -> Option<&T> {
        self.vec.last()
    }

    /// Returns a mutable pointer to the last item in the slice.
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.vec.last_mut()
    }

    /// Creates a draining iterator that removes the specified range in the
    /// vector and yields the removed items.
    pub fn drain<R>(&mut self, range: R) -> Drain<T>
    where
        R: RangeBounds<usize>,
    {
        let (start, end) = match (range.start_bound(), range.end_bound()) {
            (Bound::Included(start), Bound::Excluded(end)) => (start, end),
            _ => unreachable!("unsupported"),
        };

        self.limiter
            .borrow_mut()
            .decrease_mem(size_of::<T>() * (end - 1 - start));

        self.vec.drain(range)
    }
}

impl<T> Deref for LimitedVec<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.vec.as_slice()
    }
}

impl<T> Index<usize> for LimitedVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}

impl<T> Drop for LimitedVec<T> {
    fn drop(&mut self) {
        self.limiter
            .borrow_mut()
            .decrease_mem(size_of::<T>() * self.vec.len());
    }
}

#[cfg(test)]
mod tests {
    use super::super::MemoryLimiter;
    use super::*;
    use std::ops::Range;

    #[test]
    fn current_usage() {
        {
            let limiter = MemoryLimiter::new_shared(10);
            let mut vec_u8: LimitedVec<u8> = LimitedVec::new(limiter.clone());

            vec_u8.push(1).unwrap();
            vec_u8.push(2).unwrap();
            assert_eq!(limiter.borrow().current_usage(), 2);
        }

        {
            let limiter = MemoryLimiter::new_shared(10);
            let mut vec_u32: LimitedVec<u32> = LimitedVec::new(limiter.clone());

            vec_u32.push(1).unwrap();
            vec_u32.push(2).unwrap();
            assert_eq!(limiter.borrow().current_usage(), 8);
        }
    }

    #[test]
    fn max_limit() {
        let limiter = MemoryLimiter::new_shared(2);
        let mut vec_u8: LimitedVec<u8> = LimitedVec::new(limiter.clone());

        vec_u8.push(1).unwrap();
        vec_u8.push(2).unwrap();

        let alloc_err = vec_u8.push(3).unwrap_err();
        assert_eq!(alloc_err, ExceededLimitsError { current_usage: 3 });
    }

    #[test]
    fn drop() {
        let limiter = MemoryLimiter::new_shared(1);

        {
            let mut vec_u8: LimitedVec<u8> = LimitedVec::new(limiter.clone());
            vec_u8.push(1).unwrap();
            assert_eq!(limiter.clone().borrow().current_usage(), 1);
        }

        assert_eq!(limiter.borrow().current_usage(), 0);
    }

    #[test]
    fn drain() {
        let limiter = MemoryLimiter::new_shared(10);
        let mut vec_u8: LimitedVec<u8> = LimitedVec::new(limiter.clone());

        vec_u8.push(1).unwrap();
        vec_u8.push(2).unwrap();
        vec_u8.push(3).unwrap();
        vec_u8.drain(Range { start: 0, end: 3 });

        assert_eq!(limiter.borrow().current_usage(), 1);
    }
}

use base::{Bytes, Range};
use std::fmt::Debug;

pub trait Input<'b>: Debug + 'b {
    fn get_data(&self) -> &[u8];
    fn is_last(&self) -> bool;

    #[inline]
    fn slice(&self, range: Range) -> Bytes {
        let data = self.get_data();

        data[range.start..range.end].into()
    }

    #[inline]
    fn opt_slice(&self, range: Option<Range>) -> Option<Bytes> {
        range.map(|range| self.slice(range))
    }

    #[inline]
    fn len(&self) -> usize {
        self.get_data().len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn as_bytes(&self) -> Bytes {
        self.get_data().into()
    }

    // NOTE: slice's get() is too generic and returns a borrowed
    // value which doesn't work for us due to ownership issues
    #[inline]
    fn get(&self, pos: usize) -> Option<u8> {
        let data = self.get_data();

        if pos < data.len() {
            Some(*unsafe { data.get_unchecked(pos) })
        } else {
            None
        }
    }
}

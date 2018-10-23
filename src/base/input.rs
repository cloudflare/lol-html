use base::{Bytes, Range};
use std::fmt::Debug;
use std::ops::Deref;

pub trait Input: Debug + Deref<Target = [u8]> {
    fn get_next_pos(&self) -> usize;
    fn set_next_pos(&mut self, pos: usize);
    fn is_last(&self) -> bool;

    #[inline]
    fn slice(&self, range: Range) -> Bytes {
        self[range.start..range.end].into()
    }

    #[inline]
    fn opt_slice(&self, range: Option<Range>) -> Option<Bytes> {
        range.map(|range| self.slice(range))
    }

    // NOTE: slice's get() is too generic and returns a borrowed
    // value which doesn't work for us due to ownership issues
    #[inline]
    fn get(&self, pos: usize) -> Option<u8> {
        if pos < self.len() {
            Some(*unsafe { self.get_unchecked(pos) })
        } else {
            None
        }
    }

    #[inline]
    fn get_pos(&self) -> usize {
        self.get_next_pos() - 1
    }

    #[inline]
    fn step_back(&mut self) {
        let pos = self.get_next_pos() - 1;

        self.set_next_pos(pos);
    }

    #[inline]
    fn advance(&mut self, count: usize) {
        let pos = self.get_next_pos() + count;

        self.set_next_pos(pos);
    }

    #[inline]
    fn lookahead(&self, offset: usize) -> Option<u8> {
        self.get(self.get_next_pos() + offset - 1)
    }
}

impl Iterator for Input<Target = [u8]> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        let pos = self.get_next_pos();

        self.set_next_pos(pos + 1);

        self.get(pos)
    }
}

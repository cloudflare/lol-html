use base::{Bytes, Range};
use std::ops::Deref;

#[derive(Debug)]
pub struct IterableChunk<'b> {
    bytes: Bytes<'b>,
    next_pos: usize,
    offset_from_prev_chunk_start: usize,
    last: bool,
}

impl<'b> IterableChunk<'b> {
    pub fn new(bytes: &'b [u8], last: bool, offset_from_prev_chunk_start: usize) -> Self {
        IterableChunk {
            bytes: bytes.into(),
            next_pos: 0,
            offset_from_prev_chunk_start,
            last,
        }
    }

    #[inline]
    pub fn get_offset_from_prev_chunk_start(&self) -> usize {
        self.offset_from_prev_chunk_start
    }

    #[inline]
    pub fn slice(&self, range: Range) -> Bytes {
        self[range.start..range.end].into()
    }

    #[inline]
    pub fn maybe_slice(&self, range: Option<Range>) -> Option<Bytes> {
        range.map(|range| self.slice(range))
    }

    #[inline]
    pub fn is_last(&self) -> bool {
        self.last
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
    pub fn get_pos(&self) -> usize {
        self.next_pos - 1
    }

    #[inline]
    pub fn step_back(&mut self) {
        self.next_pos -= 1;
    }

    #[inline]
    pub fn advance(&mut self, count: usize) {
        self.next_pos += count;
    }

    #[inline]
    pub fn lookahead(&self, offset: usize) -> Option<u8> {
        self.get(self.next_pos + offset - 1)
    }
}

impl<'b> Iterator for IterableChunk<'b> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        let pos = self.next_pos;

        self.next_pos += 1;

        self.get(pos)
    }
}

impl<'b> Deref for IterableChunk<'b> {
    type Target = Bytes<'b>;

    #[inline]
    fn deref(&self) -> &Bytes<'b> {
        &self.bytes
    }
}

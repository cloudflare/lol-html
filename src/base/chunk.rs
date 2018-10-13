use base::{Bytes, Range};
use std::ops::Deref;

// TODO make a trait
#[derive(Debug)]
pub struct Chunk<'b>(Bytes<'b>);

impl<'b> From<&'b [u8]> for Chunk<'b> {
    fn from(bytes: &'b [u8]) -> Self {
        Chunk(bytes.into())
    }
}

impl<'b> Chunk<'b> {
    #[inline]
    pub fn peek_at(&self, pos: usize) -> Option<u8> {
        if pos < self.len() {
            Some(self[pos])
        } else {
            None
        }
    }

    pub fn slice(&self, range: Range) -> Bytes {
        self[range.start..range.end].into()
    }
}

impl<'b> Deref for Chunk<'b> {
    type Target = Bytes<'b>;

    #[inline]
    fn deref(&self) -> &Bytes<'b> {
        &self.0
    }
}

use std::ops::Deref;

#[cfg(feature = "testing_api")]
use std::{fmt, str};

pub struct InputChunk<'b> {
    bytes: &'b [u8],
    len: usize,
}

impl<'b> InputChunk<'b> {
    pub fn new(bytes: &'b [u8]) -> Self {
        InputChunk {
            bytes,
            len: bytes.len(),
        }
    }

    #[inline]
    pub fn peek_at(&self, pos: usize) -> Option<u8> {
        if pos < self.len {
            Some(self.bytes[pos])
        } else {
            None
        }
    }
}

impl<'b> Deref for InputChunk<'b> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.bytes
    }
}

pub struct InputChunkSlice<'c> {
    chunk: &'c InputChunk<'c>,
    start: usize,
    end: usize,
}

impl<'c> InputChunkSlice<'c> {
    pub fn new(chunk: &'c InputChunk<'c>, start: usize) -> Self {
        InputChunkSlice {
            chunk,
            start: 0,
            end: 0,
        }
    }

    #[inline]
    pub fn set_end(&mut self, end: usize) {
        self.end = end;
    }
}

impl<'c> Deref for InputChunkSlice<'c> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.chunk[self.start..self.end]
    }
}

#[cfg(feature = "testing_api")]
impl<'c> InputChunkSlice<'c> {
    pub fn as_str(&self) -> &str {
        str::from_utf8(self).unwrap()
    }

    pub fn as_string(&self) -> String {
        String::from_utf8(self.to_vec()).unwrap()
    }
}

#[cfg(feature = "testing_api")]
impl<'c> fmt::Debug for InputChunkSlice<'c> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}`", self.as_str())
    }
}

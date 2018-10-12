use std::ops::Deref;

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

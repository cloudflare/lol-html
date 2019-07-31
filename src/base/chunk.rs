use super::Bytes;

#[derive(Debug)]
pub struct Chunk<'b> {
    data: &'b [u8],
    last: bool,
    next_pos: usize,
}

impl<'b> Chunk<'b> {
    pub fn last(data: &'b [u8]) -> Self {
        Chunk {
            data,
            last: true,
            next_pos: 0,
        }
    }

    pub fn last_empty() -> Self {
        Chunk {
            data: &[],
            last: true,
            next_pos: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_last(&self) -> bool {
        self.last
    }

    #[inline]
    pub fn get(&self, pos: usize) -> Option<u8> {
        self.data.get(pos).cloned()
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.next_pos - 1
    }

    #[inline]
    pub fn set_pos(&mut self, pos: usize) {
        self.next_pos = pos;
    }

    #[inline]
    pub fn as_bytes(&self) -> Bytes<'b> {
        Bytes::from(self.data)
    }

    #[inline]
    #[allow(clippy::let_and_return)]
    pub fn consume_ch(&mut self) -> Option<u8> {
        let ch = self.get(self.next_pos);

        self.next_pos += 1;

        trace!(@chars "consume", ch);

        ch
    }

    #[inline]
    pub fn unconsume_ch(&mut self) {
        self.next_pos -= 1;

        trace!(@chars "unconsume");
    }

    #[inline]
    pub fn consume_several(&mut self, count: usize) {
        self.next_pos += count;

        trace!(@chars "consume several");
    }

    #[inline]
    #[allow(clippy::let_and_return)]
    pub fn lookahead(&self, offset: usize) -> Option<u8> {
        let ch = self.get(self.next_pos + offset - 1);

        trace!(@chars "lookahead", ch);

        ch
    }
}

impl<'b> From<&'b [u8]> for Chunk<'b> {
    fn from(data: &'b [u8]) -> Self {
        Chunk {
            data,
            last: false,
            next_pos: 0,
        }
    }
}

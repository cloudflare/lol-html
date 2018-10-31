use base::{Align, Chunk};

#[derive(Default)]
pub struct Cursor {
    next_pos: usize,
}

impl Cursor {
    #[inline]
    pub fn pos(&self) -> usize {
        self.next_pos - 1
    }

    #[inline]
    #[cfg_attr(feature = "cargo-clippy", allow(let_and_return))]
    pub fn consume_ch(&mut self, chunk: &Chunk) -> Option<u8> {
        let ch = chunk.get(self.next_pos);

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
    #[cfg_attr(feature = "cargo-clippy", allow(let_and_return))]
    pub fn lookahead(&self, chunk: &Chunk, offset: usize) -> Option<u8> {
        let ch = chunk.get(self.next_pos + offset - 1);

        trace!(@chars "lookahead", ch);

        ch
    }
}

impl Align for Cursor {
    #[inline]
    fn align(&mut self, offset: usize) {
        self.next_pos.align(offset + 1);
    }
}

use base::Input;
use std::ops::Deref;

#[derive(Debug)]
pub struct Chunk<'b> {
    data: &'b [u8],
    next_pos: usize,
    last: bool,
}

impl<'b> Chunk<'b> {
    const LAST: Chunk<'static> = Chunk {
        data: &[],
        next_pos: 0,
        last: true,
    };
}

impl<'b> From<&'b [u8]> for Chunk<'b> {
    fn from(data: &'b [u8]) -> Self {
        Chunk {
            data,
            next_pos: 0,
            last: false,
        }
    }
}

impl<'b> Input for Chunk<'b> {
    #[inline]
    fn get_next_pos(&self) -> usize {
        self.next_pos
    }

    #[inline]
    fn set_next_pos(&mut self, pos: usize) {
        self.next_pos = pos;
    }

    #[inline]
    fn is_last(&self) -> bool {
        self.last
    }
}

impl<'b> Deref for Chunk<'b> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.data
    }
}

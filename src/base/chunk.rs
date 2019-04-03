use super::{Bytes, Range};

#[derive(Debug)]
pub struct Chunk<'b> {
    data: &'b [u8],
    last: bool,
}

impl<'b> Chunk<'b> {
    pub fn last(data: &'b [u8]) -> Self {
        Chunk { data, last: true }
    }

    pub fn last_empty() -> Self {
        Chunk {
            data: &[],
            last: true,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn is_last(&self) -> bool {
        self.last
    }

    #[inline]
    pub fn slice(&self, range: Range) -> Bytes<'_> {
        self.data[range.start..range.end].into()
    }

    #[inline]
    pub fn opt_slice(&self, range: Option<Range>) -> Option<Bytes<'_>> {
        range.map(|range| self.slice(range))
    }

    #[inline]
    pub fn get(&self, pos: usize) -> Option<u8> {
        self.data.get(pos).cloned()
    }

    #[inline]
    pub(crate) fn as_debug_string(&self) -> String {
        Bytes::from(self.data).as_debug_string()
    }
}

impl<'b> From<&'b [u8]> for Chunk<'b> {
    fn from(data: &'b [u8]) -> Self {
        Chunk { data, last: false }
    }
}

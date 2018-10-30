use base::{Bytes, Range};

#[derive(Debug)]
pub struct Chunk<'b> {
    data: Bytes<'b>,
    last: bool,
}

impl<'b> Chunk<'b> {
    pub fn last(data: &'b [u8]) -> Self {
        Chunk {
            data: data.into(),
            last: true,
        }
    }

    pub fn last_empty() -> Self {
        Chunk {
            data: Bytes::empty(),
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
    pub fn slice(&self, range: Range) -> Bytes {
        self.data[range.start..range.end].into()
    }

    #[inline]
    pub fn opt_slice(&self, range: Option<Range>) -> Option<Bytes> {
        range.map(|range| self.slice(range))
    }

    #[inline]
    pub fn get(&self, pos: usize) -> Option<u8> {
        self.data.get(pos).cloned()
    }

    pub fn as_string(&self) -> String {
        self.data.as_string()
    }
}

impl<'b> From<&'b [u8]> for Chunk<'b> {
    fn from(data: &'b [u8]) -> Self {
        Chunk {
            data: data.into(),
            last: false,
        }
    }
}

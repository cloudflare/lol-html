use crate::base::{Bytes, Chunk, Range};
use std::fmt::{self, Debug};

#[derive(Debug, Copy, Clone)]
pub enum TagType {
    StartTag,
    EndTag,
}

pub struct TagHint<'i> {
    input: &'i Chunk<'i>,
    tag_type: TagType,
    name_range: Range,
    name_hash: Option<u64>,
}

impl<'i> TagHint<'i> {
    pub fn new(
        input: &'i Chunk<'i>,
        tag_type: TagType,
        name_range: Range,
        name_hash: Option<u64>,
    ) -> Self {
        TagHint {
            input,
            tag_type,
            name_range,
            name_hash,
        }
    }

    #[inline]
    pub fn name(&self) -> Bytes<'i> {
        self.input.slice(self.name_range)
    }

    #[inline]
    pub fn name_hash(&self) -> Option<u64> {
        self.name_hash
    }

    #[inline]
    pub fn tag_type(&self) -> TagType {
        self.tag_type
    }
}

impl Debug for TagHint<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TagNameInfo")
            .field("name", &self.name())
            .field("name_hash", &self.name_hash)
            .finish()
    }
}

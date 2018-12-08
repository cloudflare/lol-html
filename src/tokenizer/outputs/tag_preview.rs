use crate::base::{Bytes, Chunk, Range};
use lazycell::LazyCell;
use std::fmt::{self, Debug};

pub struct TagNameInfo<'i> {
    input: &'i Chunk<'i>,
    name_range: Range,
    name: LazyCell<Bytes<'i>>,
    pub name_hash: Option<u64>,
}

impl<'i> TagNameInfo<'i> {
    pub fn new(input: &'i Chunk<'i>, name_range: Range, name_hash: Option<u64>) -> Self {
        TagNameInfo {
            input,
            name_range,
            name: LazyCell::new(),
            name_hash,
        }
    }

    pub fn name(&self) -> &Bytes<'i> {
        self.name.borrow_with(|| self.input.slice(self.name_range))
    }
}

impl Debug for TagNameInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TagNameInfo")
            .field("name", self.name())
            .field("name_hash", &self.name_hash)
            .finish()
    }
}

#[derive(Debug)]
pub enum TagPreview<'i> {
    StartTag(TagNameInfo<'i>),
    EndTag(TagNameInfo<'i>),
}

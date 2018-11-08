use base::{Bytes, Chunk, Range};
use lazycell::LazyCell;

declare_handler! {
    TagPreviewHandler(&TagPreview)
}

pub struct TagNameInfo<'c> {
    input: &'c Chunk<'c>,
    name_range: Range,
    name: LazyCell<Bytes<'c>>,
    pub name_hash: Option<u64>,
}

impl<'c> TagNameInfo<'c> {
    pub fn new(input: &'c Chunk<'c>, name_range: Range, name_hash: Option<u64>) -> Self {
        TagNameInfo {
            input,
            name_range,
            name: LazyCell::new(),
            name_hash,
        }
    }

    pub fn get_name(&self) -> &Bytes<'c> {
        self.name.borrow_with(|| self.input.slice(self.name_range))
    }
}

pub enum TagPreview<'c> {
    StartTag(TagNameInfo<'c>),
    EndTag(TagNameInfo<'c>),
}

use super::LocalNameHash;
use crate::base::{Bytes, Chunk, Range};

/// LocalName is used for the comparison of tag names, attributes, etc.
/// In the majority of cases it will be represented as a hash, however for long
/// non-standard tag names actual bytes representation may be used.
#[derive(Clone, PartialEq, Debug)]
pub enum LocalName<'i> {
    Hash(LocalNameHash),
    Bytes(Bytes<'i>),
}

impl<'i> LocalName<'i> {
    #[inline]
    pub fn new(input: &'i Chunk<'i>, range: Range, hash: LocalNameHash) -> Self {
        if hash.is_empty() {
            LocalName::Bytes(input.slice(range))
        } else {
            LocalName::Hash(hash)
        }
    }
}

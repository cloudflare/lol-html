use super::{Lexeme, TokenOutline};
use crate::base::{Bytes, Chunk, Range};
use std::fmt::{self, Debug};

pub struct TagNameInfo<'i> {
    input: &'i Chunk<'i>,
    name_range: Range,
    name_hash: Option<u64>,
}

impl<'i> TagNameInfo<'i> {
    #[inline]
    pub fn new(input: &'i Chunk<'i>, name_range: Range, name_hash: Option<u64>) -> Self {
        TagNameInfo {
            input,
            name_range,
            name_hash,
        }
    }

    #[inline]
    pub fn from_tag_lexeme(lexeme: &'i Lexeme<'i>) -> Self {
        match lexeme.token_outline() {
            Some(TokenOutline::StartTag {
                name, name_hash, ..
            })
            | Some(TokenOutline::EndTag { name, name_hash }) => {
                TagNameInfo::new(lexeme.input(), *name, *name_hash)
            }
            _ => unreachable!("Lexeme should be a start or an end tag"),
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
}

impl Debug for TagNameInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TagNameInfo")
            .field("name", &self.name())
            .field("name_hash", &self.name_hash)
            .finish()
    }
}

use crate::parser::TextType;
use encoding_rs::Encoding;
use std::borrow::Cow;

#[derive(Debug)]
pub struct TextChunk<'i> {
    text: Cow<'i, str>,
    text_type: TextType,
    last_in_current_boundaries: bool,
    encoding: &'static Encoding,
}

impl<'i> TextChunk<'i> {
    pub(in crate::token) fn new_parsed(
        text: &'i str,
        text_type: TextType,
        last_in_current_boundaries: bool,
        encoding: &'static Encoding,
    ) -> Self {
        TextChunk {
            text: text.into(),
            text_type,
            last_in_current_boundaries,
            encoding,
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &*self.text
    }

    #[inline]
    pub fn text_type(&self) -> TextType {
        self.text_type
    }

    #[inline]
    pub fn last_in_current_boundaries(&self) -> bool {
        self.last_in_current_boundaries
    }
}

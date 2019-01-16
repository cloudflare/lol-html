use crate::parser::TextType;
use encoding_rs::Encoding;
use std::borrow::Cow;

#[derive(Debug)]
pub struct TextChunk<'i> {
    text: Cow<'i, str>,
    text_type: TextType,
    encoding: &'static Encoding,
}

impl<'i> TextChunk<'i> {
    #[inline]
    pub fn text(&self) -> &str {
        &*self.text
    }

    #[inline]
    pub fn text_type(&self) -> TextType {
        self.text_type
    }
}

#[derive(Debug)]
pub enum Text<'i> {
    Chunk(TextChunk<'i>),
    End,
}

impl<'i> Text<'i> {
    pub(crate) fn new_parsed_chunk(
        text: &'i str,
        text_type: TextType,
        encoding: &'static Encoding,
    ) -> Self {
        Text::Chunk(TextChunk {
            text: text.into(),
            text_type,
            encoding,
        })
    }
}

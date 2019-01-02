use crate::base::Bytes;
use encoding_rs::Encoding;
use std::borrow::Cow;

#[derive(Debug)]
pub struct TextChunk<'i> {
    text: Cow<'i, str>,
    encoding: &'static Encoding,
}

impl<'i> TextChunk<'i> {
    #[inline]
    pub fn text(&self) -> &str {
        &*self.text
    }

    #[inline]
    pub fn raw(&self) -> Bytes<'_> {
        self.encoding.encode(self.text()).0.into()
    }
}

#[derive(Debug)]
pub enum Text<'i> {
    Chunk(TextChunk<'i>),
    End,
}

impl<'i> Text<'i> {
    pub(crate) fn new_parsed_chunk(text: &'i str, encoding: &'static Encoding) -> Self {
        Text::Chunk(TextChunk {
            text: text.into(),
            encoding,
        })
    }
}

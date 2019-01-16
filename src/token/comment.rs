use crate::base::Bytes;
use encoding_rs::Encoding;

#[derive(Debug)]
pub struct Comment<'i> {
    text: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
}

impl<'i> Comment<'i> {
    pub(crate) fn new_parsed(text: Bytes<'i>, raw: Bytes<'i>, encoding: &'static Encoding) -> Self {
        Comment {
            text,
            raw: Some(raw),
            encoding,
        }
    }

    #[inline]
    pub fn text(&self) -> String {
        self.text.as_string(self.encoding)
    }
}

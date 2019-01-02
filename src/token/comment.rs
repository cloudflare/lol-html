use crate::base::Bytes;
use encoding_rs::Encoding;

#[derive(Getters, Debug)]
pub struct Comment<'i> {
    #[get = "pub"]
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
    pub fn raw(&self) -> Option<&Bytes<'_>> {
        self.raw.as_ref()
    }
}

use super::{Mutations, Token};
use crate::base::Bytes;
use encoding_rs::Encoding;
use std::fmt::{self, Debug};

pub struct EndTag<'i> {
    name: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
    pub mutations: Mutations,
}

impl<'i> EndTag<'i> {
    pub(super) fn new_token(
        name: Bytes<'i>,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Token<'i> {
        Token::EndTag(EndTag {
            name,
            raw: Some(raw),
            encoding,
            mutations: Mutations::new(encoding),
        })
    }

    #[inline]
    pub fn name(&self) -> String {
        self.name.as_lowercase_string(self.encoding)
    }

    #[inline]
    pub fn set_name(&mut self, name: Bytes<'static>) {
        self.name = name;
        self.raw = None;
    }

    #[inline]
    fn raw(&self) -> Option<&Bytes> {
        self.raw.as_ref()
    }

    #[inline]
    fn serialize_from_parts(&self, output_handler: &mut dyn FnMut(&[u8])) {
        output_handler(b"</");
        output_handler(&self.name);
        output_handler(b">");
    }
}

impl_serialize!(EndTag);

impl Debug for EndTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EndTag")
            .field("name", &self.name())
            .finish()
    }
}

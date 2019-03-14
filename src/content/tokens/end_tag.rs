use super::{OrderingMutations, Token};
use crate::base::Bytes;
use encoding_rs::Encoding;
use std::fmt::{self, Debug};

pub struct EndTag<'i> {
    name: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
    ordering_mutations: OrderingMutations,
}

impl<'i> EndTag<'i> {
    pub(in crate::content) fn new_token(
        name: Bytes<'i>,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Token<'i> {
        Token::EndTag(EndTag {
            name,
            raw: Some(raw),
            encoding,
            ordering_mutations: OrderingMutations::default(),
        })
    }

    #[inline]
    pub fn name(&self) -> String {
        let mut name = self.name.as_string(self.encoding);

        name.make_ascii_lowercase();

        name
    }

    #[inline]
    pub fn set_name(&mut self, name: Bytes<'static>) {
        self.name = name;
        self.raw = None;
    }

    #[inline]
    fn raw(&self) -> Option<&Bytes<'_>> {
        self.raw.as_ref()
    }

    #[inline]
    fn serialize_from_parts(&self, output_handler: &mut dyn FnMut(&[u8])) {
        output_handler(b"</");
        output_handler(&self.name);
        output_handler(b">");
    }
}

impl_common_token_api!(EndTag);

impl Debug for EndTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EndTag")
            .field("name", &self.name())
            .finish()
    }
}

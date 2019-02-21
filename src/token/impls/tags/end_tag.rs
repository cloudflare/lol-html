use super::try_tag_name_from_str;
use crate::base::Bytes;
use crate::token::{OrderingMutations, Token};
use encoding_rs::Encoding;
use failure::Error;
use std::fmt::{self, Debug};

pub struct EndTag<'i> {
    name: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
    ordering_mutations: OrderingMutations,
}

impl_common_token_api!(EndTag);

impl<'i> EndTag<'i> {
    pub(in crate::token) fn new_token(
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

    implement_tag_name_accessors!();

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

impl Debug for EndTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EndTag")
            .field("name", &self.name())
            .finish()
    }
}

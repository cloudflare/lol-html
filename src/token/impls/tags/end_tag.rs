use super::try_tag_name_from_str;
use crate::base::Bytes;
use crate::token::OrderingMutations;
use encoding_rs::Encoding;
use failure::Error;
use std::fmt::{self, Debug};

pub struct EndTag<'i> {
    name: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,

    // NOTE: we use boxed ordering mutations and lazily initialize it to not
    // increase stack size of a token with the heavy rarely used structure.
    ordering_mutations: Option<Box<OrderingMutations<'i>>>,
}

impl_common_token_api!(EndTag);

impl<'i> EndTag<'i> {
    pub(in crate::token) fn new_parsed(
        name: Bytes<'i>,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Self {
        EndTag {
            name,
            raw: Some(raw),
            encoding,
            ordering_mutations: None,
        }
    }

    pub(in crate::token) fn try_from(
        name: &str,
        encoding: &'static Encoding,
    ) -> Result<Self, Error> {
        Ok(EndTag {
            name: try_tag_name_from_str(name, encoding)?,
            raw: None,
            encoding,
            ordering_mutations: None,
        })
    }

    implement_tag_name_accessors!();

    // NOTE: not a trait implementation due to the `Borrow` constraint for
    // the `Owned` associated type.
    // See: https://github.com/rust-lang/rust/issues/44950
    #[inline]
    pub fn to_owned(&self) -> EndTag<'static> {
        EndTag {
            name: self.name.to_owned(),
            raw: Bytes::opt_to_owned(&self.raw),
            encoding: self.encoding,
            ordering_mutations: None,
        }
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

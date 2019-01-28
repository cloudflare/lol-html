use super::try_tag_name_from_str;
use crate::base::Bytes;
use crate::transform_stream::Serialize;
use encoding_rs::Encoding;
use failure::Error;
use std::fmt::{self, Debug};

pub struct EndTag<'i> {
    name: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
}

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
        }
    }
}

impl Serialize for EndTag<'_> {
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

use super::try_tag_name_from_str;
use crate::base::Bytes;
use encoding_rs::Encoding;
use failure::Error;

#[derive(Debug)]
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

    implement_tag_name_accessors!();
}

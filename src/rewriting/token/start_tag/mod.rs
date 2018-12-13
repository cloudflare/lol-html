mod attributes;

use crate::base::Bytes;
use encoding_rs::Encoding;

pub use self::attributes::*;

#[derive(Getters, Debug)]
pub struct StartTag<'i> {
    #[get = "pub"]
    name: Bytes<'i>,

    #[get = "pub"]
    attributes: Attributes<'i>,

    self_closing: bool,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
}

impl<'i> StartTag<'i> {
    pub(super) fn new_parsed(
        name: Bytes<'i>,
        attributes: ParsedAttributeList<'i>,
        self_closing: bool,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Self {
        StartTag {
            name,
            attributes: attributes.into(),
            self_closing,
            raw: Some(raw),
            encoding,
        }
    }

    #[inline]
    pub fn self_closing(&self) -> bool {
        self.self_closing
    }

    #[inline]
    pub fn raw(&self) -> Option<&Bytes<'_>> {
        self.raw.as_ref()
    }
}

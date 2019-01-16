mod attributes;

use crate::base::Bytes;
use encoding_rs::Encoding;

pub use self::attributes::*;

#[derive(Debug)]
pub struct StartTag<'i> {
    name: Bytes<'i>,
    attributes: Attributes<'i>,
    self_closing: bool,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
}

impl<'i> StartTag<'i> {
    pub(crate) fn new_parsed(
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
    pub fn attributes(&self) -> &Attributes<'i> {
        &self.attributes
    }

    #[inline]
    pub fn name(&self) -> String {
        let mut name = self.name.as_string(self.encoding);

        name.make_ascii_lowercase();

        name
    }

    #[inline]
    pub fn self_closing(&self) -> bool {
        self.self_closing
    }
}

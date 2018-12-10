mod attributes;

use crate::base::Bytes;

pub use self::attributes::*;

#[derive(Getters, Debug)]
pub struct StartTag<'i> {
    #[get = "pub"]
    name: Bytes<'i>,

    #[get = "pub"]
    attributes: Attributes<'i>,

    self_closing: bool,
}

impl<'i> StartTag<'i> {
    pub(super) fn new_parsed(
        name: Bytes<'i>,
        attributes: ParsedAttributeList<'i>,
        self_closing: bool,
    ) -> Self {
        StartTag {
            name,
            attributes: Box::new(attributes),
            self_closing,
        }
    }

    #[inline]
    pub fn self_closing(&self) -> bool {
        self.self_closing
    }
}

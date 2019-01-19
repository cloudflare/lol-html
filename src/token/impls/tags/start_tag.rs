use super::attributes::{Attribute, Attributes};
use super::try_tag_name_from_str;
use crate::base::Bytes;
use crate::transform_stream::Serialize;
use encoding_rs::Encoding;
use failure::Error;
use std::fmt::{self, Debug};

pub struct StartTag<'i> {
    name: Bytes<'i>,
    attributes: Attributes<'i>,
    self_closing: bool,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
}

impl<'i> StartTag<'i> {
    pub(in crate::token) fn new_parsed(
        name: Bytes<'i>,
        attributes: Attributes<'i>,
        self_closing: bool,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Self {
        StartTag {
            name,
            attributes,
            self_closing,
            raw: Some(raw),
            encoding,
        }
    }

    pub(in crate::token) fn try_from(
        name: &str,
        attributes: &[(&str, &str)],
        self_closing: bool,
        encoding: &'static Encoding,
    ) -> Result<Self, Error> {
        Ok(StartTag {
            name: try_tag_name_from_str(name, encoding)?,
            attributes: Attributes::try_from(attributes, encoding)?,
            self_closing,
            raw: None,
            encoding,
        })
    }

    implement_tag_name_accessors!();

    #[inline]
    pub fn attributes(&self) -> &[Attribute<'i>] {
        &*self.attributes
    }

    #[inline]
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        let name = name.to_ascii_lowercase();

        self.attributes().iter().find_map(|attr| {
            if attr.name() == name {
                Some(attr.value())
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn has_attribute(&self, name: &str) -> bool {
        let name = name.to_ascii_lowercase();

        self.attributes().iter().any(|attr| attr.name() == name)
    }

    #[inline]
    pub fn set_attribute(&mut self, name: &str, value: &str) -> Result<(), Error> {
        self.attributes.set_attribute(name, value, self.encoding)?;
        self.raw = None;

        Ok(())
    }

    #[inline]
    pub fn remove_attribute(&mut self, name: &str) {
        if self.attributes.remove_attribute(name) {
            self.raw = None;
        }
    }

    #[inline]
    pub fn self_closing(&self) -> bool {
        self.self_closing
    }

    #[inline]
    pub fn set_self_closing(&mut self, self_closing: bool) {
        self.self_closing = self_closing;
        self.raw = None;
    }

    // NOTE: not a trait implementation due to the `Borrow` constraint for
    // the `Owned` associated type.
    // See: https://github.com/rust-lang/rust/issues/44950
    #[inline]
    pub fn to_owned(&self) -> StartTag<'static> {
        StartTag {
            name: self.name.to_owned(),
            attributes: self.attributes.to_owned(),
            self_closing: self.self_closing,
            raw: self.raw.as_ref().map(|r| r.to_owned()),
            encoding: self.encoding,
        }
    }
}

impl Serialize for StartTag<'_> {
    #[inline]
    fn take_raw(&mut self) -> Option<Bytes<'_>> {
        self.raw.take()
    }

    #[inline]
    fn serialize_from_parts(self, handler: &mut dyn FnMut(Bytes<'_>)) {
        handler(b"<".into());
        handler(self.name);

        if !self.attributes.is_empty() {
            handler(b" ".into());

            self.attributes.into_bytes(handler);

            // NOTE: attributes can be modified the way that
            // last attribute has an unquoted value. We always
            // add extra space before the `/`, because otherwise
            // it will be treated as a part of such an unquotted
            // attribute value.
            if self.self_closing {
                handler(b" ".into());
            }
        }

        if self.self_closing {
            handler(b"/>".into());
        } else {
            handler(b">".into());
        }
    }
}

impl Debug for StartTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StartTag")
            .field("name", &self.name())
            .field("attributes", &self.attributes())
            .field("self_closing", &self.self_closing)
            .finish()
    }
}

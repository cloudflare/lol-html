use super::{Attribute, AttributeNameError, Attributes};
use super::{Mutations, Serialize, Token};
use crate::base::Bytes;
use encoding_rs::Encoding;
use std::fmt::{self, Debug};

pub struct StartTag<'i> {
    name: Bytes<'i>,
    attributes: Attributes<'i>,
    self_closing: bool,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
    pub mutations: Mutations,
}

impl<'i> StartTag<'i> {
    pub(super) fn new_token(
        name: Bytes<'i>,
        attributes: Attributes<'i>,
        self_closing: bool,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Token<'i> {
        Token::StartTag(StartTag {
            name,
            attributes,
            self_closing,
            raw: Some(raw),
            encoding,
            mutations: Mutations::new(encoding),
        })
    }

    #[inline]
    pub fn encoding(&self) -> &'static Encoding {
        self.encoding
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
    pub fn attributes(&self) -> &[Attribute<'i>] {
        &*self.attributes
    }

    #[inline]
    pub fn set_attribute(&mut self, name: &str, value: &str) -> Result<(), AttributeNameError> {
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
    #[cfg(feature = "test_api")]
    pub fn self_closing(&self) -> bool {
        self.self_closing
    }

    #[inline]
    #[cfg(feature = "test_api")]
    pub fn set_self_closing(&mut self, self_closing: bool) {
        self.self_closing = self_closing;
        self.raw = None;
    }

    #[inline]
    fn raw(&self) -> Option<&Bytes> {
        self.raw.as_ref()
    }

    #[inline]
    fn serialize_from_parts(&self, output_handler: &mut dyn FnMut(&[u8])) {
        output_handler(b"<");
        output_handler(&self.name);

        if !self.attributes.is_empty() {
            output_handler(b" ");

            self.attributes.to_bytes(output_handler);

            // NOTE: attributes can be modified the way that
            // last attribute has an unquoted value. We always
            // add extra space before the `/`, because otherwise
            // it will be treated as a part of such an unquotted
            // attribute value.
            if self.self_closing {
                output_handler(b" ");
            }
        }

        if self.self_closing {
            output_handler(b"/>");
        } else {
            output_handler(b">");
        }
    }

    #[cfg(feature = "test_api")]
    pub fn raw_attributes(
        &self,
    ) -> (
        &'i crate::base::Chunk<'i>,
        crate::parser::SharedAttributeBuffer,
    ) {
        self.attributes.raw_attributes()
    }
}

impl_serialize!(StartTag);

impl Debug for StartTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StartTag")
            .field("name", &self.name())
            .field("attributes", &self.attributes())
            .field("self_closing", &self.self_closing)
            .finish()
    }
}

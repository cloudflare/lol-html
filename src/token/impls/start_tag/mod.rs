mod attributes;

use crate::base::Bytes;
use crate::transform_stream::Serialize;
use encoding_rs::Encoding;
use failure::Error;

pub use self::attributes::AttributeNameError;
pub(in crate::token) use self::attributes::{Attribute, Attributes, ParsedAttributeList};

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum TagNameError {
    #[fail(display = "Tag name can't be empty.")]
    Empty,
    #[fail(display = "First character of the tag name should be an ASCII alphabetical character.")]
    InvalidFirstCharacter,
    #[fail(display = "{:?} character is forbidden in the tag name", _0)]
    ForbiddenCharacter(char),
    #[fail(display = "The tag name contains a character that can't \
                      be represented in the document's character encoding.")]
    UnencodableCharacter,
}

#[derive(Debug)]
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
            name: StartTag::name_from_str(name, encoding)?,
            attributes: Attributes::try_from(attributes, encoding)?,
            self_closing,
            raw: None,
            encoding,
        })
    }

    fn name_from_str(name: &str, encoding: &'static Encoding) -> Result<Bytes<'static>, Error> {
        match name.chars().nth(0) {
            Some(ch) if !ch.is_ascii_alphabetic() => {
                Err(TagNameError::InvalidFirstCharacter.into())
            }
            Some(_) => {
                if let Some(ch) = name.chars().find(|&ch| match ch {
                    ' ' | '\n' | '\r' | '\t' | '\x0C' | '/' | '>' => true,
                    _ => false,
                }) {
                    Err(TagNameError::ForbiddenCharacter(ch).into())
                } else {
                    // NOTE: if character can't be represented in the given
                    // encoding then encoding_rs replaces it with a numeric
                    // character reference. Character references are not
                    // supported in attribute names, so we need to bail.
                    match Bytes::from_str_without_replacements(name, encoding) {
                        Some(name) => Ok(name.into_owned()),
                        None => Err(TagNameError::UnencodableCharacter.into()),
                    }
                }
            }
            None => Err(TagNameError::Empty.into()),
        }
    }

    #[inline]
    pub fn name(&self) -> String {
        let mut name = self.name.as_string(self.encoding);

        name.make_ascii_lowercase();

        name
    }

    #[inline]
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        self.name = StartTag::name_from_str(name, self.encoding)?;
        self.raw = None;

        Ok(())
    }

    #[inline]
    pub fn attributes(&self) -> &[Attribute<'i>] {
        &*self.attributes
    }

    #[inline]
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes.get_attribute(name)
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

    // NOTE: not a trait a implementation due to the `Borrow` constraint for
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

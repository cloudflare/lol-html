use super::{Attribute, AttributeNameError, ContentType, StartTag};
use crate::base::Bytes;
use encoding_rs::Encoding;

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

pub struct Element<'r, 't> {
    start_tag: &'r mut StartTag<'t>,
    encoding: &'static Encoding,
}

impl<'r, 't> Element<'r, 't> {
    pub(crate) fn new(start_tag: &'r mut StartTag<'t>) -> Self {
        let encoding = start_tag.encoding();

        Element {
            start_tag,
            encoding,
        }
    }

    fn tag_name_bytes_from_str(&self, name: &str) -> Result<Bytes<'static>, TagNameError> {
        match name.chars().nth(0) {
            Some(ch) if !ch.is_ascii_alphabetic() => Err(TagNameError::InvalidFirstCharacter),
            Some(_) => {
                if let Some(ch) = name.chars().find(|&ch| match ch {
                    ' ' | '\n' | '\r' | '\t' | '\x0C' | '/' | '>' => true,
                    _ => false,
                }) {
                    Err(TagNameError::ForbiddenCharacter(ch))
                } else {
                    // NOTE: if character can't be represented in the given
                    // encoding then encoding_rs replaces it with a numeric
                    // character reference. Character references are not
                    // supported in tag names, so we need to bail.
                    match Bytes::from_str_without_replacements(name, self.encoding) {
                        Some(name) => Ok(name.into_owned()),
                        None => Err(TagNameError::UnencodableCharacter),
                    }
                }
            }
            None => Err(TagNameError::Empty),
        }
    }

    #[inline]
    pub fn tag_name(&self) -> String {
        self.start_tag.name()
    }

    #[inline]
    pub fn set_tag_name(&mut self, name: &str) -> Result<(), TagNameError> {
        let name = self.tag_name_bytes_from_str(name)?;

        self.start_tag.set_name(name);

        Ok(())
    }

    #[inline]
    pub fn attributes(&self) -> &[Attribute<'t>] {
        self.start_tag.attributes()
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
    pub fn set_attribute(&mut self, name: &str, value: &str) -> Result<(), AttributeNameError> {
        self.start_tag.set_attribute(name, value)
    }

    #[inline]
    pub fn remove_attribute(&mut self, name: &str) {
        self.start_tag.remove_attribute(name);
    }

    #[inline]
    pub fn set_text(&mut self, _text: &str) {
        unimplemented!();
    }

    #[inline]
    pub fn set_inner_html(&mut self, _html: &str) {
        unimplemented!();
    }

    #[inline]
    pub fn insert_before(&mut self, content: &str, content_type: ContentType) {
        self.start_tag.insert_before(content, content_type);
    }

    #[inline]
    pub fn insert_after(&mut self, _content: &str, _content_type: ContentType) {
        unimplemented!()
    }

    #[inline]
    pub fn prepend(&mut self, content: &str, content_type: ContentType) {
        self.start_tag.insert_after(content, content_type);
    }

    #[inline]
    pub fn append(&mut self, _content: &str, _content_type: ContentType) {
        unimplemented!()
    }

    #[inline]
    pub fn replace(&mut self, _content: &str, _content_type: ContentType) {
        unimplemented!()
    }

    #[inline]
    pub fn remove(&mut self, _content: &str, _content_type: ContentType) {
        unimplemented!()
    }

    #[inline]
    pub fn remove_and_keep_content(&mut self, _content: &str, _content_type: ContentType) {
        unimplemented!()
    }

    #[inline]
    pub fn removed(&mut self) -> bool {
        unimplemented!()
    }
}

#[cfg(feature = "test_api")]
pub fn create_element<'r, 't>(start_tag: &'r mut StartTag<'t>) -> Element<'r, 't> {
    Element::new(start_tag)
}

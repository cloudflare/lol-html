use super::{Attribute, AttributeNameError, ContentType, EndTag, Mutations, StartTag};
use crate::base::Bytes;
use crate::rewriter::EndTagHandler;
use encoding_rs::Encoding;
use std::fmt::{self, Debug};

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
    end_tag_mutations: Option<Mutations>,
    modified_tag_name: Option<Bytes<'static>>,
    encoding: &'static Encoding,
}

impl<'r, 't> Element<'r, 't> {
    pub(crate) fn new(start_tag: &'r mut StartTag<'t>) -> Self {
        let encoding = start_tag.encoding();

        Element {
            start_tag,
            end_tag_mutations: None,
            modified_tag_name: None,
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
                        Ok(name) => Ok(name.into_owned()),
                        Err(_) => Err(TagNameError::UnencodableCharacter),
                    }
                }
            }
            None => Err(TagNameError::Empty),
        }
    }

    #[inline]
    fn end_tag_mutations_mut(&mut self) -> &mut Mutations {
        let encoding = self.encoding;

        self.end_tag_mutations
            .get_or_insert_with(|| Mutations::new(encoding))
    }

    #[inline]
    pub fn tag_name(&self) -> String {
        self.start_tag.name()
    }

    #[inline]
    pub fn set_tag_name(&mut self, name: &str) -> Result<(), TagNameError> {
        let name = self.tag_name_bytes_from_str(name)?;

        self.modified_tag_name = Some(name.clone());
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
    pub fn before(&mut self, content: &str, content_type: ContentType) {
        self.start_tag.before(content, content_type);
    }

    #[inline]
    pub fn after(&mut self, content: &str, content_type: ContentType) {
        self.end_tag_mutations_mut().after(content, content_type);
    }

    #[inline]
    pub fn prepend(&mut self, content: &str, content_type: ContentType) {
        self.start_tag.after(content, content_type);
    }

    #[inline]
    pub fn append(&mut self, content: &str, content_type: ContentType) {
        self.end_tag_mutations_mut().before(content, content_type);
    }

    #[inline]
    pub fn set_inner_content(&mut self, content: &str, content_type: ContentType) {
        unimplemented!();
    }

    #[inline]
    pub fn replace(&mut self, _content: &str, _content_type: ContentType) {
        unimplemented!()
    }

    #[inline]
    pub fn remove(&mut self) {
        unimplemented!()
    }

    #[inline]
    pub fn remove_and_keep_content(&mut self) {
        unimplemented!()
    }

    #[inline]
    pub fn removed(&self) -> bool {
        self.start_tag.removed()
    }

    pub(crate) fn into_end_tag_handler(self) -> Option<EndTagHandler<'static>> {
        let end_tag_mutations = self.end_tag_mutations;
        let modified_tag_name = self.modified_tag_name;

        if end_tag_mutations.is_some() || modified_tag_name.is_some() {
            // TODO: remove this hack when Box<dyn FnOnce> become callable in Rust 1.35.
            let mut wrap = Some(move |end_tag: &mut EndTag| {
                if let Some(name) = modified_tag_name {
                    end_tag.set_name(name);
                }

                if let Some(mutations) = end_tag_mutations {
                    end_tag.set_mutations(mutations);
                }
            });

            Some(Box::new(move |end_tag| {
                (wrap.take().expect("FnOnce called more than once"))(end_tag)
            }))
        } else {
            None
        }
    }
}

impl Debug for Element<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Element")
            .field("tag_name", &self.tag_name())
            .field("attributes", &self.attributes())
            .finish()
    }
}

use crate::base::Bytes;
use encoding_rs::Encoding;
use failure::Error;

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

fn try_tag_name_from_str(name: &str, encoding: &'static Encoding) -> Result<Bytes<'static>, Error> {
    match name.chars().nth(0) {
        Some(ch) if !ch.is_ascii_alphabetic() => Err(TagNameError::InvalidFirstCharacter.into()),
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
                // supported in tag names, so we need to bail.
                match Bytes::from_str_without_replacements(name, encoding) {
                    Some(name) => Ok(name.into_owned()),
                    None => Err(TagNameError::UnencodableCharacter.into()),
                }
            }
        }
        None => Err(TagNameError::Empty.into()),
    }
}

macro_rules! implement_tag_name_accessors {
    () => {
        #[inline]
        pub fn name(&self) -> String {
            let mut name = self.name.as_string(self.encoding);

            name.make_ascii_lowercase();

            name
        }

        #[inline]
        pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
            self.name = try_tag_name_from_str(name, self.encoding)?;
            self.raw = None;

            Ok(())
        }
    };
}

mod attributes;
mod end_tag;
mod start_tag;

pub(in crate::token) use self::attributes::Attributes;

pub use self::attributes::{Attribute, AttributeNameError};
pub use self::end_tag::EndTag;
pub use self::start_tag::StartTag;

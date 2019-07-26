use super::{Mutations, Token};
use crate::base::Bytes;
use crate::html::TextType;
use encoding_rs::Encoding;
use std::any::Any;
use std::borrow::Cow;
use std::fmt::{self, Debug};

pub struct TextChunk<'i> {
    text: Cow<'i, str>,
    text_type: TextType,
    last_in_text_node: bool,
    encoding: &'static Encoding,
    mutations: Mutations,
    user_data: Box<dyn Any>,
}

impl<'i> TextChunk<'i> {
    pub(super) fn new_token(
        text: &'i str,
        text_type: TextType,
        last_in_text_node: bool,
        encoding: &'static Encoding,
    ) -> Token<'i> {
        Token::TextChunk(TextChunk {
            text: text.into(),
            text_type,
            last_in_text_node,
            encoding,
            mutations: Mutations::new(encoding),
            user_data: Box::new(()),
        })
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &*self.text
    }

    #[inline]
    pub fn text_type(&self) -> TextType {
        self.text_type
    }

    #[inline]
    pub fn last_in_text_node(&self) -> bool {
        self.last_in_text_node
    }

    #[inline]
    fn raw(&self) -> Option<&Bytes> {
        None
    }

    #[inline]
    fn serialize_from_parts(&self, output_handler: &mut dyn FnMut(&[u8])) {
        if !self.text.is_empty() {
            output_handler(&Bytes::from_str(&self.text, self.encoding));
        }
    }
}

inject_mutation_api!(TextChunk);
impl_serialize!(TextChunk);
impl_user_data!(TextChunk<'_>);

impl Debug for TextChunk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TextChunk")
            .field("text", &self.as_str())
            .field("last_in_text_node", &self.last_in_text_node())
            .finish()
    }
}

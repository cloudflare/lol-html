use super::Token;

pub enum Content<'i> {
    Token(Token<'i>),
    Html(&'i str),
}

impl<'i> From<&'i str> for Content<'i> {
    fn from(html: &'i str) -> Self {
        Content::Html(html)
    }
}

macro_rules! impl_from_token {
    ($($Token:ident),+) => {
        $(
            impl<'i> From<$Token<'i>> for Content<'i> {
                fn from(token: $Token<'i>) -> Self {
                    Content::Token(token.into())
                }
            }
        )+
    };
}

impl_from_token!(TextChunk, Comment, StartTag, EndTag, Doctype);

#[derive(Default)]
pub struct OrderingMutations<'i> {
    content_before: Vec<Content<'i>>,
    replacements: Vec<Content<'i>>,
    content_after: Vec<Content<'i>>,
    removed: bool,
}

macro_rules! impl_common_token_api {
    ($Token:ident) => {
        impl<'i> $Token<'i> {
            #[inline]
            fn get_ordering_mutations_mut(&mut self) -> &mut OrderingMutations<'i> {
                self.ordering_mutations
                    .get_or_insert_with(|| Box::new(OrderingMutations::default()))
            }

            #[inline]
            pub fn before(&mut self, content: crate::token::Content<'i>) {
                self.get_ordering_mutations_mut()
                    .content_before
                    .push(content);
            }

            #[inline]
            pub fn after(&mut self, content: crate::token::Content<'i>) {
                self.get_ordering_mutations_mut()
                    .content_after
                    .insert(0, content);;
            }

            #[inline]
            pub fn replace(&mut self, content: crate::token::Content<'i>) {
                self.get_ordering_mutations_mut().replacements.push(content);
            }

            #[inline]
            pub fn remove(&mut self) {
                self.get_ordering_mutations_mut().removed = true;
            }

            #[inline]
            fn serialize(&self, output_handler: &mut dyn FnMut(&[u8])) {
                match self.raw.as_ref() {
                    Some(raw) => output_handler(raw),
                    None => self.serialize_from_parts(output_handler),
                }
            }

            #[inline]
            fn serialize_content_list(
                &self,
                list: &[crate::token::Content<'_>],
                output_handler: &mut dyn FnMut(&[u8]),
            ) {
                use crate::token::{Content, Serialize};

                for item in list {
                    match item {
                        Content::Token(t) => t.to_bytes(output_handler),
                        Content::Html(s) => {
                            let html = self.encoding.encode(s).0;

                            output_handler(&html);
                        }
                    }
                }
            }
        }

        impl crate::token::Serialize for $Token<'_> {
            #[inline]
            fn to_bytes(&self, output_handler: &mut dyn FnMut(&[u8])) {
                match self.ordering_mutations.as_ref() {
                    Some(m) => {
                        self.serialize_content_list(&m.content_before, output_handler);

                        if !m.replacements.is_empty() {
                            self.serialize_content_list(&m.replacements, output_handler);
                        } else if !m.removed {
                            self.serialize(output_handler);
                        }

                        self.serialize_content_list(&m.content_after, output_handler);
                    }
                    None => self.serialize(output_handler),
                }
            }
        }
    };
}

mod comment;
mod doctype;
mod tags;
mod text_chunk;

pub use self::comment::{Comment, CommentTextError};
pub use self::doctype::Doctype;
pub use self::tags::*;
pub use self::text_chunk::{TextChunk, TextError};

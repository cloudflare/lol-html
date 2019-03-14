#[derive(Default)]
struct OrderingMutations {
    content_before: Vec<u8>,
    replacements: Vec<u8>,
    content_after: Vec<u8>,
    removed: bool,
}

macro_rules! impl_common_token_api {
    ($Token:ident) => {
        impl<'i> $Token<'i> {
            #[inline]
            pub fn insert_before(
                &mut self,
                content: &str,
                content_type: crate::content::ContentType,
            ) {
                crate::content::content_to_bytes(content, content_type, self.encoding, &mut |c| {
                    self.ordering_mutations.content_before.extend_from_slice(c)
                });
            }

            #[inline]
            pub fn insert_after(
                &mut self,
                content: &str,
                content_type: crate::content::ContentType,
            ) {
                let mut pos = 0;

                crate::content::content_to_bytes(content, content_type, self.encoding, &mut |c| {
                    self.ordering_mutations
                        .content_after
                        .splice(pos..pos, c.iter().cloned());

                    pos += c.len();
                });
            }

            #[inline]
            pub fn replace(&mut self, content: &str, content_type: crate::content::ContentType) {
                crate::content::content_to_bytes(content, content_type, self.encoding, &mut |c| {
                    self.ordering_mutations.replacements.extend_from_slice(c)
                });

                self.ordering_mutations.removed = true;
            }

            #[inline]
            pub fn remove(&mut self) {
                self.ordering_mutations.removed = true;
            }

            #[inline]
            pub fn removed(&self) -> bool {
                self.ordering_mutations.removed
            }
        }

        impl crate::content::Serialize for $Token<'_> {
            #[inline]
            fn to_bytes(&self, output_handler: &mut dyn FnMut(&[u8])) {
                let OrderingMutations {
                    content_before,
                    replacements,
                    content_after,
                    removed,
                } = &self.ordering_mutations;

                if !content_before.is_empty() {
                    output_handler(content_before);
                }

                if !removed {
                    match self.raw() {
                        Some(raw) => output_handler(raw),
                        None => self.serialize_from_parts(output_handler),
                    }
                } else if !replacements.is_empty() {
                    output_handler(replacements);
                }

                if !content_after.is_empty() {
                    output_handler(content_after);
                }
            }
        }
    };
}

mod attributes;
mod comment;
mod doctype;
mod end_tag;
mod start_tag;
mod text_chunk;

pub(super) use self::attributes::Attributes;

pub use self::attributes::{Attribute, AttributeNameError};
pub use self::comment::{Comment, CommentTextError};
pub use self::doctype::Doctype;
pub use self::end_tag::EndTag;
pub use self::start_tag::StartTag;
pub use self::text_chunk::TextChunk;

pub trait Serialize {
    fn to_bytes(&self, output_handler: &mut dyn FnMut(&[u8]));
}

#[derive(Debug)]
pub enum Token<'i> {
    TextChunk(TextChunk<'i>),
    Comment(Comment<'i>),
    StartTag(StartTag<'i>),
    EndTag(EndTag<'i>),
    Doctype(Doctype<'i>),
}

impl Serialize for Token<'_> {
    #[inline]
    fn to_bytes(&self, output_handler: &mut dyn FnMut(&[u8])) {
        match self {
            Token::TextChunk(t) => t.to_bytes(output_handler),
            Token::Comment(t) => t.to_bytes(output_handler),
            Token::StartTag(t) => t.to_bytes(output_handler),
            Token::EndTag(t) => t.to_bytes(output_handler),
            Token::Doctype(t) => t.to_bytes(output_handler),
        }
    }
}

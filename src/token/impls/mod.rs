#[derive(Default)]
pub struct OrderingMutations {
    content_before: Vec<u8>,
    replacements: Vec<u8>,
    content_after: Vec<u8>,
    removed: bool,
}

macro_rules! impl_common_token_api {
    ($Token:ident) => {
        impl<'i> $Token<'i> {
            #[inline]
            pub fn before(&mut self, html: &str) {
                let encoding = self.encoding;

                self.ordering_mutations
                    .content_before
                    .extend_from_slice(&crate::base::Bytes::from_str(html, encoding));
            }

            #[inline]
            pub fn after(&mut self, html: &str) {
                let encoding = self.encoding;

                self.ordering_mutations.content_after.splice(
                    0..0,
                    crate::base::Bytes::from_str(html, encoding).iter().cloned(),
                );
            }

            #[inline]
            pub fn replace(&mut self, html: &str) {
                let encoding = self.encoding;

                self.ordering_mutations
                    .replacements
                    .extend_from_slice(&crate::base::Bytes::from_str(html, encoding));
            }

            #[inline]
            pub fn remove(&mut self) {
                self.ordering_mutations.removed = true;
            }
        }

        impl crate::token::Serialize for $Token<'_> {
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

                if !replacements.is_empty() {
                    output_handler(replacements);
                } else if !removed {
                    match self.raw() {
                        Some(raw) => output_handler(raw),
                        None => self.serialize_from_parts(output_handler),
                    }
                }

                if !content_after.is_empty() {
                    output_handler(content_after);
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
pub use self::text_chunk::TextChunk;

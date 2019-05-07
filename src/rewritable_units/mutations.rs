use crate::rewritable_units::{content_to_bytes, ContentType};
use encoding_rs::Encoding;

pub struct Mutations {
    pub content_before: Vec<u8>,
    pub replacements: Vec<u8>,
    pub content_after: Vec<u8>,
    pub removed: bool,
    encoding: &'static Encoding,
}

impl Mutations {
    #[inline]
    pub fn new(encoding: &'static Encoding) -> Self {
        Mutations {
            content_before: Vec::default(),
            replacements: Vec::default(),
            content_after: Vec::default(),
            removed: false,
            encoding,
        }
    }

    #[inline]
    pub fn before(&mut self, content: &str, content_type: ContentType) {
        content_to_bytes(content, content_type, self.encoding, &mut |c| {
            self.content_before.extend_from_slice(c);
        });
    }

    #[inline]
    pub fn after(&mut self, content: &str, content_type: ContentType) {
        let mut pos = 0;

        content_to_bytes(content, content_type, self.encoding, &mut |c| {
            self.content_after.splice(pos..pos, c.iter().cloned());

            pos += c.len();
        });
    }

    #[inline]
    pub fn replace(&mut self, content: &str, content_type: ContentType) {
        content_to_bytes(content, content_type, self.encoding, &mut |c| {
            self.replacements.extend_from_slice(c)
        });

        self.remove();
    }

    #[inline]
    pub fn remove(&mut self) {
        self.removed = true;
    }

    #[inline]
    pub fn removed(&self) -> bool {
        self.removed
    }
}

macro_rules! inject_mutation_api {
    ($Token:ident) => {
        impl<'i> $Token<'i> {
            #[inline]
            pub fn before(
                &mut self,
                content: &str,
                content_type: crate::rewritable_units::ContentType,
            ) {
                self.mutations.before(content, content_type);
            }

            #[inline]
            pub fn after(
                &mut self,
                content: &str,
                content_type: crate::rewritable_units::ContentType,
            ) {
                self.mutations.after(content, content_type);
            }

            #[inline]
            pub fn replace(
                &mut self,
                content: &str,
                content_type: crate::rewritable_units::ContentType,
            ) {
                self.mutations.replace(content, content_type);
            }

            #[inline]
            pub fn remove(&mut self) {
                self.mutations.remove();
            }

            #[inline]
            pub fn removed(&self) -> bool {
                self.mutations.removed()
            }
        }
    };
}

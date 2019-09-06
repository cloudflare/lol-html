use crate::base::Bytes;
use encoding_rs::Encoding;

/// The type of inserted content.
pub enum ContentType {
    /// HTML content type. The rewriter will insert the content as is.
    Html,
    /// Text content type. The rewriter will HTML-escape the content before insertion:
    ///     - `<` will be replaced with `&lt;`
    ///     - `>` will be replaced with `&gt;`
    ///     - `&` will be replaced with `&amp;`
    Text,
}

#[inline]
fn content_to_bytes(
    content: &str,
    content_type: ContentType,
    encoding: &'static Encoding,
    output_handler: &mut dyn FnMut(&[u8]),
) {
    let bytes = Bytes::from_str(content, encoding);

    match content_type {
        ContentType::Html => output_handler(&bytes),
        ContentType::Text => bytes.replace_byte3(
            (b'<', b"&lt;"),
            (b'>', b"&gt;"),
            (b'&', b"&amp;"),
            output_handler,
        ),
    }
}

pub struct Mutations {
    pub content_before: Vec<u8>,
    pub replacement: Vec<u8>,
    pub content_after: Vec<u8>,
    pub removed: bool,
    encoding: &'static Encoding,
}

impl Mutations {
    #[inline]
    pub fn new(encoding: &'static Encoding) -> Self {
        Mutations {
            content_before: Vec::default(),
            replacement: Vec::default(),
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
        let mut replacement = Vec::default();

        content_to_bytes(content, content_type, self.encoding, &mut |c| {
            replacement.extend_from_slice(c);
        });

        self.replacement = replacement;
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
    ($Token:ident, $doc_name:expr) => {
        use doc_comment::doc_comment;

        impl<'i> $Token<'i> {
            doc_comment! {
                concat![
                    "Inserts `content` before the ", $doc_name, ".\n",
                    "\n",
                    "Consequent calls to the method append `content` to the previously ",
                    "inserted content."
                ],

                #[inline]
                pub fn before(
                    &mut self,
                    content: &str,
                    content_type: crate::rewritable_units::ContentType,
                ) {
                    self.mutations.before(content, content_type);
                }
            }

            doc_comment! {
                concat![
                    "Inserts content after the ", $doc_name, ".\n",
                    "\n",
                    "Consequent calls to the method prepend `content` to the previously ",
                    "inserted content."
                ],

                #[inline]
                pub fn after(
                    &mut self,
                    content: &str,
                    content_type: crate::rewritable_units::ContentType,
                ) {
                    self.mutations.after(content, content_type);
                }
            }

            doc_comment! {
                concat![
                    "Replaces the ", $doc_name, " with the `content`.\n",
                    "\n",
                    "Consequent calls to the method overwrite previous replacement content."
                ],

                #[inline]
                pub fn replace(
                    &mut self,
                    content: &str,
                    content_type: crate::rewritable_units::ContentType,
                ) {
                    self.mutations.replace(content, content_type);
                }
            }

            doc_comment! {
                concat![
                    "Removes the ", $doc_name, "."
                ],

                #[inline]
                pub fn remove(&mut self) {
                    self.mutations.remove();
                }
            }

            doc_comment! {
                concat![
                    "Returns `true` if the ", $doc_name, " has been removed by calling ",
                    "[`replace`] or [`remove`].\n",
                    "\n",
                    "[`replace`]: #method.replace\n",
                    "[`remove`]: #method.remove\n",
                ],

                #[inline]
                pub fn removed(&self) -> bool {
                    self.mutations.removed()
                }
            }
        }
    };
}

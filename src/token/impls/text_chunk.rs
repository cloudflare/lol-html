use crate::base::Bytes;
use crate::parser::TextType;
use crate::transform_stream::Serialize;
use encoding_rs::Encoding;
use std::borrow::Cow;

#[derive(Debug)]
pub struct TextChunk<'i> {
    text: Cow<'i, str>,
    text_type: TextType,
    last_in_current_boundaries: bool,
    requires_escape_on_output: bool,
    encoding: &'static Encoding,
}

impl<'i> TextChunk<'i> {
    pub(in crate::token) fn new(
        text: &'i str,
        text_type: TextType,
        last_in_current_boundaries: bool,
        encoding: &'static Encoding,
    ) -> Self {
        TextChunk {
            text: text.into(),
            text_type,
            last_in_current_boundaries,
            requires_escape_on_output: false,
            encoding,
        }
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
    pub fn last_in_current_boundaries(&self) -> bool {
        self.last_in_current_boundaries
    }

    #[inline]
    pub fn escape_on_output(&mut self) {
        self.requires_escape_on_output = true;
    }

    // NOTE: not a trait implementation due to the `Borrow` constraint for
    // the `Owned` associated type.
    // See: https://github.com/rust-lang/rust/issues/44950
    #[inline]
    pub fn to_owned(&self) -> TextChunk<'static> {
        TextChunk {
            text: Cow::Owned(self.text.to_string()),
            text_type: self.text_type,
            last_in_current_boundaries: self.last_in_current_boundaries,
            requires_escape_on_output: self.requires_escape_on_output,
            encoding: self.encoding,
        }
    }
}

impl Serialize for TextChunk<'_> {
    #[inline]
    fn raw(&self) -> Option<&Bytes<'_>> {
        None
    }

    #[inline]
    fn serialize_from_parts(&self, output_handler: &mut dyn FnMut(&[u8])) {
        if !self.text.is_empty() {
            let text = self.encoding.encode(&self.text).0;

            if self.requires_escape_on_output {
                Bytes::from(text).replace_byte3(
                    (b'<', b"&lt;"),
                    (b'>', b"&gt;"),
                    (b'&', b"&amp;"),
                    output_handler,
                );
            } else {
                output_handler(&text);
            }
        }
    }
}

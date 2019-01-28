use crate::base::Bytes;
use crate::parser::TextType;
use crate::transform_stream::Serialize;
use encoding_rs::Encoding;
use failure::Error;
use std::borrow::Cow;

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum TextError {
    #[fail(display = "Script text shouldn't contain `</script>` end tag.")]
    ScriptEndTagInScriptText,
    #[fail(display = "Stylesheet text shouldn't contain `</style>` end tag.")]
    StyleEndTagInStylesheetText,
    #[fail(
        display = "Text contains a character that can't be represented in the \
                   document's character encoding."
    )]
    UnencodableCharacter,
}

#[derive(Debug)]
pub struct TextChunk<'i> {
    text: Cow<'i, str>,
    raw: Option<Bytes<'i>>,
    text_type: TextType,
    last_in_current_boundaries: bool,
    parsed: bool,
    encoding: &'static Encoding,
}

impl<'i> TextChunk<'i> {
    pub(in crate::token) fn new_parsed(
        text: &'i str,
        text_type: TextType,
        last_in_current_boundaries: bool,
        encoding: &'static Encoding,
    ) -> Self {
        TextChunk {
            text: text.into(),
            raw: None,
            text_type,
            last_in_current_boundaries,
            parsed: true,
            encoding,
        }
    }

    pub(in crate::token) fn new(text: &'i str, encoding: &'static Encoding) -> Self {
        TextChunk {
            text: text.into(),
            raw: None,
            text_type: TextType::Data,
            last_in_current_boundaries: false,
            parsed: false,
            encoding,
        }
    }

    #[inline]
    fn try_element_specific_text_from(
        text: &'i str,
        encoding: &'static Encoding,
        closing_tag: &'static str,
        closing_tag_error: TextError,
        text_type: TextType,
    ) -> Result<Self, Error> {
        if text.find(closing_tag).is_some() {
            Err(closing_tag_error.into())
        } else {
            // NOTE: both `<script>` and `<style>` doesn't allow text entities
            // in their text, so unencodable characters are not allowed.
            // Since we perform encoding for the validation anyway and chunk
            // content is immutable, we store encoded bytes as `raw`, so
            // they later can be used during serialization.
            match Bytes::from_str_without_replacements(text, encoding) {
                Some(raw) => Ok(TextChunk {
                    text: text.into(),
                    text_type,
                    last_in_current_boundaries: false,
                    raw: Some(raw),
                    parsed: false,
                    encoding,
                }),
                None => Err(TextError::UnencodableCharacter.into()),
            }
        }
    }

    pub(in crate::token) fn try_script_from(
        text: &'i str,
        encoding: &'static Encoding,
    ) -> Result<Self, Error> {
        Self::try_element_specific_text_from(
            text,
            encoding,
            "</script>",
            TextError::ScriptEndTagInScriptText,
            TextType::ScriptData,
        )
    }

    pub(in crate::token) fn try_stylesheet_from(
        text: &'i str,
        encoding: &'static Encoding,
    ) -> Result<Self, Error> {
        Self::try_element_specific_text_from(
            text,
            encoding,
            "</style>",
            TextError::StyleEndTagInStylesheetText,
            TextType::RawText,
        )
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &*self.text
    }

    #[inline]
    #[cfg(feature = "testing_api")]
    pub fn text_type(&self) -> TextType {
        self.text_type
    }

    #[inline]
    pub fn last_in_current_boundaries(&self) -> bool {
        self.last_in_current_boundaries
    }

    // NOTE: not a trait implementation due to the `Borrow` constraint for
    // the `Owned` associated type.
    // See: https://github.com/rust-lang/rust/issues/44950
    #[inline]
    pub fn to_owned(&self) -> TextChunk<'static> {
        TextChunk {
            text: Cow::Owned(self.text.to_string()),
            raw: Bytes::opt_to_owned(&self.raw),
            text_type: self.text_type,
            last_in_current_boundaries: self.last_in_current_boundaries,
            parsed: self.parsed,
            encoding: self.encoding,
        }
    }
}

impl Serialize for TextChunk<'_> {
    #[inline]
    fn raw(&self) -> Option<&Bytes<'_>> {
        self.raw.as_ref()
    }

    #[inline]
    fn serialize_from_parts(&self, output_handler: &mut dyn FnMut(&[u8])) {
        if !self.text.is_empty() {
            let text = self.encoding.encode(&self.text).0;

            if !self.parsed && self.text_type.allows_text_entitites() {
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

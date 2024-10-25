use crate::base::Bytes;
use encoding_rs::Encoding;
use std::error::Error as StdError;

type BoxResult = Result<(), Box<dyn StdError + Send + Sync>>;

/// The type of inserted content.
#[derive(Copy, Clone)]
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
pub(super) fn content_to_bytes(
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
            &mut *output_handler,
        ),
    }
}

pub(crate) struct Mutations {
    pub content_before: DynamicString,
    pub replacement: DynamicString,
    pub content_after: DynamicString,
    pub removed: bool,
    pub encoding: &'static Encoding,
}

impl Mutations {
    #[inline]
    #[must_use]
    pub const fn new(encoding: &'static Encoding) -> Self {
        Self {
            content_before: DynamicString::new(),
            replacement: DynamicString::new(),
            content_after: DynamicString::new(),
            removed: false,
            encoding,
        }
    }

    #[inline]
    pub fn replace(&mut self, chunk: StringChunk) {
        self.remove();
        self.replacement.clear();
        self.replacement.push_back(chunk);
    }

    #[inline]
    pub fn remove(&mut self) {
        self.removed = true;
    }

    #[inline]
    pub const fn removed(&self) -> bool {
        self.removed
    }
}

impl From<(&str, ContentType)> for StringChunk {
    #[inline]
    fn from((content, content_type): (&str, ContentType)) -> Self {
        Self::Buffer(Box::from(content), content_type)
    }
}

pub(crate) enum StringChunk {
    Buffer(Box<str>, ContentType),
}

#[derive(Default)]
pub(crate) struct DynamicString {
    chunks: Vec<StringChunk>,
}

impl DynamicString {
    #[inline]
    pub const fn new() -> Self {
        Self { chunks: vec![] }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    #[inline]
    pub fn push_front(&mut self, chunk: StringChunk) {
        self.chunks.insert(0, chunk);
    }

    #[inline]
    pub fn push_back(&mut self, chunk: StringChunk) {
        self.chunks.push(chunk);
    }

    pub fn into_bytes(
        self,
        encoding: &'static Encoding,
        output_handler: &mut dyn FnMut(&[u8]),
    ) -> BoxResult {
        for chunk in self.chunks {
            match chunk {
                StringChunk::Buffer(content, content_type) => {
                    content_to_bytes(&content, content_type, encoding, output_handler);
                }
            };
        }
        Ok(())
    }
}

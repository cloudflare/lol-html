use super::StreamingHandlerSink;
use encoding_rs::Encoding;
use std::error::Error as StdError;
use std::panic::{RefUnwindSafe, UnwindSafe};

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
    Stream(Box<dyn StreamingHandler>),
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

    pub fn encode(self, sink: &mut StreamingHandlerSink<'_>) -> BoxResult {
        for chunk in self.chunks {
            match chunk {
                StringChunk::Buffer(content, content_type) => {
                    sink.write_str(&content, content_type);
                }
                StringChunk::Stream(handler) => {
                    handler.write_all(sink)?;
                }
            };
        }
        Ok(())
    }
}

/// A callback used to write content asynchronously.
pub trait StreamingHandler: Send {
    /// This method is called only once, and is expected to write content
    /// by calling the [`sink.write_str()`](StreamingHandlerSink::write_str) one or more times.
    ///
    /// Multiple calls to `sink.write_str()` append more content to the output.
    ///
    /// See [`StreamingHandlerSink`].
    fn write_all(self: Box<Self>, sink: &mut StreamingHandlerSink<'_>) -> BoxResult;

    // Safety: due to lack of Sync, this trait must not have `&self` methods
}

/// Avoid requring `StreamingHandler` to be `Sync`.
/// It only has a method taking exclusive ownership, so there's no sharing possible.
unsafe impl Sync for StringChunk {}
impl RefUnwindSafe for StringChunk {}
impl UnwindSafe for StringChunk {}

impl<F> From<F> for Box<dyn StreamingHandler>
where
    F: FnOnce(&mut StreamingHandlerSink<'_>) -> BoxResult + Send + 'static,
{
    #[inline]
    fn from(f: F) -> Self {
        Box::new(f)
    }
}

impl<F> StreamingHandler for F
where
    F: FnOnce(&mut StreamingHandlerSink<'_>) -> BoxResult + Send + 'static,
{
    #[inline]
    fn write_all(self: Box<F>, sink: &mut StreamingHandlerSink<'_>) -> BoxResult {
        (self)(sink)
    }
}

impl From<Box<dyn StreamingHandler>> for StringChunk {
    #[inline]
    fn from(writer: Box<dyn StreamingHandler>) -> Self {
        Self::Stream(writer)
    }
}

use super::text_encoder::StreamingHandlerSink;
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

pub(crate) struct MutationsInner {
    pub content_before: DynamicString,
    pub replacement: DynamicString,
    pub content_after: DynamicString,
    pub removed: bool,
}

impl MutationsInner {
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
}

pub(crate) struct Mutations {
    inner: Option<Box<MutationsInner>>,
}

impl Mutations {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { inner: None }
    }

    #[inline]
    pub fn take(&mut self) -> Option<Box<MutationsInner>> {
        self.inner.take()
    }

    #[inline]
    pub fn if_mutated(&mut self) -> Option<&mut MutationsInner> {
        self.inner.as_deref_mut()
    }

    #[inline]
    pub fn mutate(&mut self) -> &mut MutationsInner {
        #[inline(never)]
        fn alloc_content(inner: &mut Option<Box<MutationsInner>>) -> &mut MutationsInner {
            inner.get_or_insert_with(move || {
                Box::new(MutationsInner {
                    content_before: DynamicString::new(),
                    replacement: DynamicString::new(),
                    content_after: DynamicString::new(),
                    removed: false,
                })
            })
        }

        match &mut self.inner {
            Some(inner) => inner,
            uninit => alloc_content(uninit),
        }
    }

    #[inline]
    pub fn removed(&self) -> bool {
        self.inner.as_ref().is_some_and(|inner| inner.removed)
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

    pub fn encode(self, sink: &mut StreamingHandlerSink<'_>) -> BoxResult {
        for chunk in self.chunks {
            match chunk {
                StringChunk::Buffer(content, content_type) => {
                    sink.write_str(&content, content_type);
                }
            };
        }
        Ok(())
    }
}

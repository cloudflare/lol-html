use super::mutations::content_to_bytes;
use super::ContentType;

use encoding_rs::Encoding;

use crate::transform_stream::OutputSink;

/// A rewritable unit that represents the end of the document.
///
/// This exposes the [append](#method.append) function that can be used to append content at the
/// end of the document. The content will only be appended after the rewriter has finished processing
/// the final chunk.
pub struct DocumentEnd<'a> {
    output_sink: &'a mut dyn OutputSink,
    encoding: &'static Encoding,
}

impl<'a> DocumentEnd<'a> {
    pub(crate) fn new(output_sink: &'a mut dyn OutputSink, encoding: &'static Encoding) -> Self {
        DocumentEnd {
            output_sink,
            encoding,
        }
    }

    /// Appends `content` at the end of the document.
    ///
    /// Subsequent calls to this method append `content` to the previously inserted content.
    ///
    /// # Example
    ///
    /// ```
    /// use lol_html::{end, rewrite_str, RewriteStrSettings};
    /// use lol_html::html_content::{ContentType, DocumentEnd};
    ///
    /// let html = rewrite_str(
    ///     r#"<div id="foo"><!-- content --></div><img>"#,
    ///     RewriteStrSettings {
    ///         document_content_handlers: vec![end!(|end| {
    ///             end.append("<bar>", ContentType::Html);
    ///             end.append("<baz>", ContentType::Text);
    ///             Ok(())
    ///         })],
    ///         ..RewriteStrSettings::default()
    ///     }
    /// ).unwrap();
    ///
    /// assert_eq!(html, r#"<div id="foo"><!-- content --></div><img><bar>&lt;baz&gt;"#);
    /// ```
    #[inline]
    pub fn append(&mut self, content: &str, content_type: ContentType) {
        content_to_bytes(content, content_type, self.encoding, &mut |c: &[u8]| {
            self.output_sink.handle_chunk(c)
        });
    }
}

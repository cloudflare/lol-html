use super::{ContentType, StreamingHandlerSink};
use crate::transform_stream::OutputSink;
use encoding_rs::Encoding;

/// A rewritable unit that represents the moment the rewriter is about to abandon
/// processing through a graceful bail-out.
///
/// Bail-out handlers registered via [`Settings::append_bail_out_handler()`] receive a
/// `&mut BailOut` and can emit final bytes into the output sink via [`append()`]. This
/// is the only opportunity for content other handlers have buffered (e.g. text withheld
/// pending a future chunk) to land in the response when the rewriter aborts.
///
/// Bytes appended via this unit are written *before* the rewriter's own raw flush of
/// remaining unparsed input. The resulting sink order is:
///
/// 1. Transformed bytes the rewriter already emitted normally.
/// 2. Bytes appended by bail-out handlers, in registration order.
/// 3. The rewriter's raw flush of the chunk's unparsed suffix.
///
/// [`Settings::append_bail_out_handler()`]:
///     crate::Settings::append_bail_out_handler
/// [`append()`]: Self::append
pub struct BailOut<'a> {
    output_sink: &'a mut dyn OutputSink,
    encoding: &'static Encoding,
}

impl<'a> BailOut<'a> {
    #[inline]
    #[must_use]
    pub(crate) fn new(output_sink: &'a mut dyn OutputSink, encoding: &'static Encoding) -> Self {
        Self {
            output_sink,
            encoding,
        }
    }

    /// Appends `content` at the bail-out point.
    ///
    /// Subsequent calls to this method append `content` to the previously inserted
    /// content within the same bail-out invocation. When multiple bail-out handlers are
    /// registered, their `append` calls are concatenated in registration order.
    ///
    /// `content_type` controls how the content is interpreted before being written to
    /// the sink. See [`ContentType`].
    ///
    /// # Example
    ///
    /// ```
    /// use lol_html::{bail_out, Settings};
    /// use lol_html::errors::RewritingError;
    /// use lol_html::html_content::ContentType;
    ///
    /// // A handler that, on content-handler-error bail-out, drops a notice into the sink
    /// // before the rewriter's own raw flush of remaining unparsed input.
    /// let settings = Settings::new()
    ///     .with_graceful_bail_out_on_content_handler_error(true)
    ///     .append_bail_out_handler(bail_out!(|err, bail_out| {
    ///         if matches!(err, RewritingError::ContentHandlerError(_)) {
    ///             bail_out.append("<!-- bailed out -->", ContentType::Html);
    ///         }
    ///     }));
    /// # let _ = settings;
    /// ```
    #[inline]
    pub fn append(&mut self, content: &str, content_type: ContentType) {
        StreamingHandlerSink::new(self.encoding, &mut |c| {
            self.output_sink.handle_chunk(c);
        })
        .write_str(content, content_type);
    }
}

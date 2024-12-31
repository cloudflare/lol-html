use crate::base::SharedEncoding;
use crate::rewriter::RewritingError;
use encoding_rs::{CoderResult, Decoder, Encoding};

pub(crate) struct TextDecoder {
    encoding: SharedEncoding,
    pending_text_streaming_decoder: Option<Decoder>,
    text_buffer: String,
}

impl TextDecoder {
    #[inline]
    #[must_use]
    pub fn new(encoding: SharedEncoding) -> Self {
        Self {
            encoding,
            pending_text_streaming_decoder: None,
            // TODO make adjustable
            text_buffer: String::from_utf8(vec![0u8; 1024]).unwrap(),
        }
    }

    #[inline]
    pub fn flush_pending(
        &mut self,
        output_handler: &mut dyn FnMut(&str, bool, &'static Encoding) -> Result<(), RewritingError>,
    ) -> Result<(), RewritingError> {
        if self.pending_text_streaming_decoder.is_some() {
            self.feed_text(&[], true, output_handler)?;
            self.pending_text_streaming_decoder = None;
        }
        Ok(())
    }

    #[inline(never)]
    pub fn feed_text(
        &mut self,
        mut raw_input: &[u8],
        last_in_text_node: bool,
        output_handler: &mut dyn FnMut(&str, bool, &'static Encoding) -> Result<(), RewritingError>,
    ) -> Result<(), RewritingError> {
        let encoding = self.encoding.get();
        let buffer = self.text_buffer.as_mut_str();

        let decoder = self
            .pending_text_streaming_decoder
            .get_or_insert_with(|| encoding.new_decoder_without_bom_handling());

        loop {
            let (status, read, written, ..) =
                decoder.decode_to_str(raw_input, buffer, last_in_text_node);

            let finished_decoding = status == CoderResult::InputEmpty;

            if written > 0 || last_in_text_node {
                // the last call to feed_text() may make multiple calls to output_handler,
                // but only one call to output_handler can be *the* last one.
                let really_last = last_in_text_node && finished_decoding;
                (output_handler)(&buffer[..written], really_last, encoding)?;
            }

            if finished_decoding {
                return Ok(());
            }
            raw_input = &raw_input[read..];
        }
    }
}

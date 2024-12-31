use crate::base::SharedEncoding;
use crate::html::TextType;
use crate::html_content::TextChunk;
use crate::rewriter::RewritingError;
use encoding_rs::{CoderResult, Decoder};

pub(crate) struct TextDecoder {
    encoding: SharedEncoding,
    pending_text_streaming_decoder: Option<Decoder>,
    text_buffer: String,
    last_text_type: TextType,
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
            last_text_type: TextType::Data,
        }
    }

    #[inline]
    pub fn flush_pending(
        &mut self,
        event_handler: &mut dyn FnMut(TextChunk<'_>) -> Result<(), RewritingError>,
    ) -> Result<(), RewritingError> {
        if self.pending_text_streaming_decoder.is_some() {
            self.decode_with_streaming_decoder(&[], true, event_handler)?;
            self.pending_text_streaming_decoder = None;
        }
        Ok(())
    }

    #[inline(never)]
    fn decode_with_streaming_decoder(
        &mut self,
        mut raw_input: &[u8],
        last: bool,
        event_handler: &mut dyn FnMut(TextChunk<'_>) -> Result<(), RewritingError>,
    ) -> Result<(), RewritingError> {
        let encoding = self.encoding.get();
        let buffer = self.text_buffer.as_mut_str();

        let decoder = self
            .pending_text_streaming_decoder
            .get_or_insert_with(|| encoding.new_decoder_without_bom_handling());

        loop {
            let (status, read, written, ..) = decoder.decode_to_str(&raw_input, buffer, last);

            if written > 0 || last {
                (event_handler)(TextChunk::new(
                    &buffer[..written],
                    self.last_text_type,
                    last,
                    self.encoding.get(),
                ))?;
            }

            if status == CoderResult::InputEmpty {
                break;
            }

            raw_input = &raw_input[read..];
        }

        Ok(())
    }

    #[inline]
    pub fn feed_text(
        &mut self,
        raw: &[u8],
        text_type: TextType,
        event_handler: &mut dyn FnMut(TextChunk<'_>) -> Result<(), RewritingError>,
    ) -> Result<(), RewritingError> {
        self.last_text_type = text_type;
        self.decode_with_streaming_decoder(raw, false, event_handler)
    }
}

mod dispatcher;

use self::dispatcher::Dispatcher;
pub use self::dispatcher::OutputSink;
pub(crate) use self::dispatcher::{AuxStartTagInfo, DispatcherError};
pub use self::dispatcher::{StartTagHandlingResult, TransformController};
use crate::AsciiCompatibleEncoding;
use crate::base::SharedEncoding;
use crate::memory::{Arena, SharedMemoryLimiter};
use crate::parser::{Parser, ParserDirective};
use crate::rewriter::RewritingError;

// Pub only for integration tests
pub struct TransformStreamSettings<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    pub transform_controller: C,
    pub output_sink: O,
    pub preallocated_parsing_buffer_size: usize,
    pub memory_limiter: SharedMemoryLimiter,
    pub encoding: AsciiCompatibleEncoding,
    pub next_encoding: SharedEncoding,
    pub strict: bool,
    pub graceful_bail_out_on_memory_limit_exceeded: bool,
    pub graceful_bail_out_on_content_handler_error: bool,
}

// Pub only for integration tests
pub struct TransformStream<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    parser: Parser<Dispatcher<C, O>>,
    buffer: Arena,
    has_buffered_data: bool,
    graceful_bail_out_on_memory_limit_exceeded: bool,
    graceful_bail_out_on_content_handler_error: bool,
}

impl<C, O> TransformStream<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    pub fn new(settings: TransformStreamSettings<C, O>) -> Self {
        let initial_parser_directive = if settings
            .transform_controller
            .initial_capture_flags()
            .is_empty()
        {
            ParserDirective::WherePossibleScanForTagsOnly
        } else {
            ParserDirective::Lex
        };

        let dispatcher = Dispatcher::new(
            settings.transform_controller,
            settings.output_sink,
            settings.encoding,
            settings.next_encoding,
        );

        let buffer = Arena::new(
            settings.memory_limiter,
            settings.preallocated_parsing_buffer_size,
        );

        let parser = Parser::new(dispatcher, initial_parser_directive, settings.strict);

        Self {
            parser,
            buffer,
            has_buffered_data: false,
            graceful_bail_out_on_memory_limit_exceeded: settings
                .graceful_bail_out_on_memory_limit_exceeded,
            graceful_bail_out_on_content_handler_error: settings
                .graceful_bail_out_on_content_handler_error,
        }
    }

    /// Returns whether the current settings allow bailing out gracefully on `err`. Memory and
    /// content-handler errors are gated by independent flags; parsing-ambiguity errors are
    /// never recovered from (the whole point of strict mode is to refuse uncertain markup).
    fn should_bail_out_for(&self, err: &RewritingError) -> bool {
        match err {
            RewritingError::MemoryLimitExceeded(_) => {
                self.graceful_bail_out_on_memory_limit_exceeded
            }
            RewritingError::ContentHandlerError(_) => {
                self.graceful_bail_out_on_content_handler_error
            }
            RewritingError::ParsingAmbiguity(_) => false,
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), RewritingError> {
        trace!(@write data);

        let chunk = if self.has_buffered_data {
            match self.buffer.append(data) {
                Ok(()) => self.buffer.bytes(),
                Err(e) => {
                    // We can't fit `data` next to the buffered (still-unparsed) bytes from
                    // previous calls. Neither chunk has been emitted to the sink yet, so on a
                    // graceful bail-out we flush both as-is and let the caller continue the
                    // response from where they were.
                    if self.graceful_bail_out_on_memory_limit_exceeded {
                        let dispatcher = self.parser.get_dispatcher();
                        dispatcher.flush_for_bail_out(self.buffer.bytes());
                        dispatcher.flush_for_bail_out(data);
                    }

                    return Err(RewritingError::MemoryLimitExceeded(e));
                }
            }
        } else {
            data
        };

        trace!(@chunk chunk);

        let consumed_byte_count = match self.parser.parse(chunk, false) {
            Ok(c) => c,
            Err(e) => {
                // The parser failed mid-chunk. The dispatcher's `remaining_content_start`
                // points to the first byte of `chunk` that hasn't been emitted yet (memory
                // errors happen before `lexeme_consumed()`; content handler errors happen
                // between `emit_chunk_before_lexeme()` and `consume_lexeme()`). Flushing from
                // there preserves all bytes the caller fed us.
                if self.should_bail_out_for(&e) {
                    self.parser.get_dispatcher().flush_for_bail_out(chunk);
                }

                return Err(e);
            }
        };

        self.parser
            .get_dispatcher()
            .flush_remaining_input(chunk, consumed_byte_count);

        if consumed_byte_count < chunk.len() {
            if self.has_buffered_data {
                self.buffer.shift(consumed_byte_count);
            } else if let Some(unconsumed) = data.get(consumed_byte_count..) {
                if let Err(e) = self.buffer.init_with(unconsumed) {
                    // Parsing succeeded but we can't buffer the leftover bytes for the next
                    // call. On a graceful bail-out we flush the leftover raw so the response
                    // stays whole.
                    if self.graceful_bail_out_on_memory_limit_exceeded {
                        self.parser.get_dispatcher().flush_for_bail_out(unconsumed);
                    }

                    return Err(RewritingError::MemoryLimitExceeded(e));
                }

                self.has_buffered_data = true;
            } else {
                debug_assert!(false);
            }
        } else {
            self.has_buffered_data = false;
        }

        Ok(())
    }

    pub fn end(&mut self) -> Result<(), RewritingError> {
        trace!(@end);

        let chunk = if self.has_buffered_data {
            self.buffer.bytes()
        } else {
            &[]
        };

        trace!(@chunk chunk);

        if let Err(e) = self.parser.parse(chunk, true) {
            // Same reasoning as in `write()`: if we can bail out gracefully, make sure the sink
            // has all the input bytes before propagating the error.
            if self.should_bail_out_for(&e) {
                self.parser.get_dispatcher().flush_for_bail_out(chunk);
            }

            return Err(e);
        }

        // `finish()` flushes any remaining input *first* and only then calls `handle_end()`,
        // so a `ContentHandlerError` from the end handler arrives after the sink already has
        // every input byte. No additional flush needed; the caller continues from where the
        // rewriter left off.
        self.parser.get_dispatcher().finish(chunk)
    }

    #[cfg(feature = "_integration_test")]
    #[allow(private_interfaces)]
    pub fn parser(&mut self) -> &mut Parser<Dispatcher<C, O>> {
        &mut self.parser
    }
}

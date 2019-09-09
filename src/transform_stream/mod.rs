mod dispatcher;

use self::dispatcher::Dispatcher;
use crate::base::Chunk;
use crate::memory::{Arena, SharedMemoryLimiter};

use crate::parser::{Parser, ParserDirective, SharedAttributeBuffer};
use encoding_rs::Encoding;
use failure::{Error, ResultExt};
use std::cell::RefCell;
use std::rc::Rc;

pub use self::dispatcher::{
    AuxStartTagInfo, DispatcherError, OutputSink, StartTagHandlingResult, TransformController,
};

const BUFFER_ERROR_CONTEXT: &str = concat!(
    "This is caused by the parser encountering an extremely long ",
    "tag or a comment that is captured by the specified selector."
);

pub struct TransformStreamSettings<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    pub transform_controller: C,
    pub output_sink: O,
    pub preallocated_memory: usize,
    pub memory_limiter: SharedMemoryLimiter,
    pub encoding: &'static Encoding,
    pub strict: bool,
}

pub struct TransformStream<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    dispatcher: Rc<RefCell<Dispatcher<C, O>>>,
    parser: Parser<Dispatcher<C, O>>,
    buffer: Arena,
    has_buffered_data: bool,
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

        let dispatcher = Rc::new(RefCell::new(Dispatcher::new(
            settings.transform_controller,
            settings.output_sink,
            settings.encoding,
        )));

        let buffer = Arena::new(settings.memory_limiter, settings.preallocated_memory);
        let parser = Parser::new(&dispatcher, initial_parser_directive, settings.strict);

        TransformStream {
            dispatcher,
            parser,
            buffer,
            has_buffered_data: false,
        }
    }

    fn buffer_blocked_bytes(
        &mut self,
        data: &[u8],
        blocked_byte_count: usize,
    ) -> Result<(), Error> {
        if self.has_buffered_data {
            self.buffer.shrink_to_last(blocked_byte_count);
        } else {
            let blocked_bytes = &data[data.len() - blocked_byte_count..];

            self.buffer
                .init_with(blocked_bytes)
                .context(BUFFER_ERROR_CONTEXT)?;

            self.has_buffered_data = true;
        }

        trace!(@buffer self.buffer);

        Ok(())
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        trace!(@write data);

        let chunk = if self.has_buffered_data {
            self.buffer.append(data).context(BUFFER_ERROR_CONTEXT)?;
            self.buffer.bytes()
        } else {
            data
        }
        .into();

        trace!(@chunk chunk);

        let blocked_byte_count = self.parser.parse(&chunk)?;

        self.dispatcher
            .borrow_mut()
            .flush_remaining_input(&chunk, blocked_byte_count);

        if blocked_byte_count > 0 {
            self.buffer_blocked_bytes(data, blocked_byte_count)?;
        } else {
            self.has_buffered_data = false;
        }

        Ok(())
    }

    pub fn end(&mut self) -> Result<(), Error> {
        trace!(@end);

        let chunk = if self.has_buffered_data {
            Chunk::last(self.buffer.bytes())
        } else {
            Chunk::last_empty()
        };

        trace!(@chunk chunk);

        self.parser.parse(&chunk)?;
        self.dispatcher.borrow_mut().finish(&chunk);

        Ok(())
    }

    #[cfg(feature = "integration_test")]
    pub fn parser(&mut self) -> &mut Parser<Dispatcher<C, O>> {
        &mut self.parser
    }
}

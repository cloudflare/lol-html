mod token_factory;

use crate::base::{Buffer, Chunk};
use crate::tokenizer::{OutputSink as TokenizerOutputSink, Tokenizer};
use failure::{Error, ResultExt};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "testing_api")]
use crate::tokenizer::Tokenizer;

const BUFFER_ERROR_CONTEXT: &str = concat!(
    "This is caused by the parser encountering an extremely long ",
    "tag or a comment that is captured by the specified selector."
);

#[derive(Fail, Debug)]
pub enum TransformStreamError {
    #[fail(display = "Data was written into the stream after it has ended.")]
    WriteCallAfterEnd,
    #[fail(display = "Stream was ended twice.")]
    EndCallAfterEnd,
}

pub struct TransformStream<S: TokenizerOutputSink> {
    tokenizer: Tokenizer<S>,
    buffer: Buffer,
    has_buffered_data: bool,
    finished: bool,
}

impl<S: TokenizerOutputSink> TransformStream<S> {
    pub fn new(buffer_capacity: usize, tokenizer_output_sink: S) -> Self {
        let tokenizer_output_sink = Rc::new(RefCell::new(tokenizer_output_sink));

        TransformStream {
            tokenizer: Tokenizer::new(&tokenizer_output_sink),
            buffer: Buffer::new(buffer_capacity),
            has_buffered_data: false,
            finished: false,
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
        if self.finished {
            return Err(TransformStreamError::WriteCallAfterEnd.into());
        }

        trace!(@write data);

        let chunk = if self.has_buffered_data {
            self.buffer.append(data).context(BUFFER_ERROR_CONTEXT)?;
            self.buffer.bytes()
        } else {
            data
        }
        .into();

        trace!(@chunk chunk);

        let blocked_byte_count = self.tokenizer.tokenize(&chunk)?;

        if blocked_byte_count > 0 {
            self.buffer_blocked_bytes(data, blocked_byte_count)?;
        } else {
            self.has_buffered_data = false;
        }

        Ok(())
    }

    pub fn end(&mut self) -> Result<(), Error> {
        if self.finished {
            return Err(TransformStreamError::EndCallAfterEnd.into());
        }

        self.finished = true;
        trace!(@end);

        let chunk = if self.has_buffered_data {
            Chunk::last(self.buffer.bytes())
        } else {
            Chunk::last_empty()
        };

        trace!(@chunk chunk);

        self.tokenizer.tokenize(&chunk)?;

        Ok(())
    }

    #[cfg(feature = "testing_api")]
    pub fn tokenizer(&mut self) -> &mut Tokenizer<LUH, TLUH, PH> {
        self.tokenizer
    }
}

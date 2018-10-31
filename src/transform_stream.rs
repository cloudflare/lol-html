use base::{Buffer, Chunk};
use errors::Error;
use tokenizer::{LexUnitHandler, Tokenizer};

pub struct TransformStream<H: LexUnitHandler> {
    tokenizer: Tokenizer<H>,
    buffer: Buffer,
    has_buffered_data: bool,
    finished: bool,
}

impl<H: LexUnitHandler> TransformStream<H> {
    pub fn new(buffer_capacity: usize, lex_unit_handler: H) -> Self {
        TransformStream {
            tokenizer: Tokenizer::new(lex_unit_handler),
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

            self.buffer.init_with(blocked_bytes)?;
            self.has_buffered_data = true;
        }

        trace!(@buffer self.buffer);

        Ok(())
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        assert!(!self.finished, "Attempt to call write() after end()");
        trace!(@write data);

        let blocked_byte_count = {
            let chunk = if self.has_buffered_data {
                self.buffer.append(data)?;
                self.buffer.bytes()
            } else {
                data
            }.into();

            trace!(@chunk chunk);

            self.tokenizer.tokenize(&chunk)?
        };

        if blocked_byte_count > 0 {
            self.buffer_blocked_bytes(data, blocked_byte_count)?;
        } else {
            self.has_buffered_data = false;
        }

        Ok(())
    }

    pub fn end(&mut self) -> Result<(), Error> {
        assert!(!self.finished, "Attempt to call end() twice");
        trace!(@end);

        self.finished = true;

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
    pub fn get_tokenizer(&mut self) -> &mut Tokenizer<H> {
        &mut self.tokenizer
    }
}

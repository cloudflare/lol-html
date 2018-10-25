use base::{Buffer, Chunk};
use errors::Error;
use tokenizer::{LexUnitHandler, Tokenizer};

pub struct TransformStream<H> {
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

    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        assert!(!self.finished, "Attempt to call write() after end()");

        let blocked_byte_count = if self.has_buffered_data {
            self.buffer.append(data)?;
            self.tokenizer.tokenize(&self.buffer)
        } else {
            self.tokenizer.tokenize(&Chunk::from(data))
        }?;

        let need_to_buffer = blocked_byte_count > 0;

        if need_to_buffer {
            if self.has_buffered_data {
                // TODO: trace for buffering
                // TODO: debug for input can be removed after debug for starttag
                self.buffer.shrink_to_last(blocked_byte_count);
            } else {
                let blocked_bytes = &data[data.len() - blocked_byte_count..];

                self.buffer.init_with(blocked_bytes)?;
            }
        }

        self.has_buffered_data = need_to_buffer;

        Ok(())
    }

    pub fn end(&mut self) -> Result<(), Error> {
        assert!(!self.finished, "Attempt to call end() twice");

        self.finished = true;

        if self.has_buffered_data {
            self.buffer.mark_as_last_input();
            self.tokenizer.tokenize(&self.buffer)
        } else {
            self.tokenizer.tokenize(&Chunk::last())
        }?;

        Ok(())
    }

    #[cfg(feature = "testing_api")]
    pub fn get_tokenizer(&mut self) -> &mut Tokenizer<H> {
        &mut self.tokenizer
    }
}

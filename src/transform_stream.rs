use base::{Buffer, IterableChunk};
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

    // 1. align to after loop in tokenizer
    // 2. tokenizer return blocked
    // 3. buffer.shrink_to
    // 4. set buffered chunk position to blocked

    pub fn write(&mut self, chunk: &[u8]) -> Result<(), Error> {
        assert!(!self.finished, "Attempt to call write() after end()");

        let blocked_byte_count = {
            let mut chunk = IterableChunk::new(
                if self.has_buffered_data {
                    self.buffer.append(chunk)?;
                    &self.buffer
                } else {
                    chunk
                },
                false,
            );

            self.tokenizer.tokenize_chunk(&mut chunk)?
        };

        if blocked_byte_count > 0 {
            if self.has_buffered_data {
                self.buffer.shrink_to_last(blocked_byte_count);
            } else {
                let blocked_bytes = &chunk[chunk.len() - blocked_byte_count..];

                self.buffer.init_with(blocked_bytes)?;
            }
        }

        Ok(())
    }

    pub fn end(&mut self) -> Result<(), Error> {
        assert!(!self.finished, "Attempt to call end() twice");

        self.finished = true;

        self.tokenizer.tokenize_chunk(&mut IterableChunk::new(
            if self.has_buffered_data {
                &self.buffer
            } else {
                &[]
            },
            true,
        ))?;

        Ok(())
    }

    #[cfg(feature = "testing_api")]
    pub fn get_tokenizer(&mut self) -> &mut Tokenizer<H> {
        &mut self.tokenizer
    }
}

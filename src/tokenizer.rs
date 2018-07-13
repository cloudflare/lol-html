pub enum Token {
    Eof,
}

pub struct BufferCapacityExceededError<'c> {
    unprocessed_buffer: &'c [u8],
}

pub struct Tokenizer<'t> {
    buffer: Box<[u8]>,
    buffer_capacity: usize,
    buffer_watermark: usize,
    token_handler: &'t FnMut(&Token),
    state: fn(&mut Tokenizer<'t>),
}

impl<'t> Tokenizer<'t> {
    fn new(token_handler: &FnMut(&Token), buffer_capacity: usize) -> Tokenizer {
        Tokenizer {
            buffer: Vec::with_capacity(buffer_capacity).into_boxed_slice(),
            buffer_capacity,
            buffer_watermark: 0,
            token_handler,
            state: Tokenizer::data_state,
        }
    }

    fn write(&mut self, chunk: Vec<u8>) -> Result<(), BufferCapacityExceededError> {
        let chunk_len = chunk.len();

        if self.buffer_watermark + chunk_len > self.buffer_capacity {
            return Err(BufferCapacityExceededError {
                unprocessed_buffer: &self.buffer[0..self.buffer_watermark],
            });
        }

        let new_watermark = self.buffer_watermark + chunk_len;

        (&mut self.buffer[self.buffer_watermark..new_watermark]).copy_from_slice(&chunk);
        self.buffer_watermark = new_watermark;

        Ok(())
    }

    fn data_state(&mut self) {}
}

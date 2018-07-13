pub struct BufferCapacityExceededError<'c> {
    unprocessed_chunk: &'c [u8],
}

pub struct Tokenizer {
    buffer: Box<[u8]>,
    buffer_capacity: usize,
    buffer_watermark: usize,
}

impl Tokenizer {
    fn new(buffer_capacity: usize) -> Tokenizer {
        Tokenizer {
            buffer: Vec::with_capacity(buffer_capacity).into_boxed_slice(),
            buffer_capacity,
            buffer_watermark: 0,
        }
    }

    fn write(&mut self, chunk: Vec<u8>) -> Result<(), BufferCapacityExceededError> {
        let chunk_len = chunk.len();

        if self.buffer_watermark + chunk_len > self.buffer_capacity {
            return Err(BufferCapacityExceededError {
                unprocessed_chunk: &self.buffer[0..self.buffer_watermark],
            });
        }

        Ok(())
    }
}

use std::ops::Deref;
use tokenizer::TokenizerBailoutReason;

pub struct InputChunk {
    bytes: Box<[u8]>,
    capacity: usize,
    watermark: usize,
}

impl InputChunk {
    pub fn new(capacity: usize) -> Self {
        InputChunk {
            bytes: vec![0; capacity].into(),
            capacity,
            watermark: 0,
        }
    }

    // 2. Rename write in tokenizer into tokenize_chunk
    // 3. Rename end into finish in Tokenizer
    // 4. Don't store InputChunk in tokenizer
    // 5. Get rid of token view, since we don't have referential
    // structure problem anymore.

    // TransformStream contains tokenizer and input
    // Peek is implemented on input, tokenizer methods
    // receive input as argument.

    // Data type:
    // 1. Original chunk
    // 2. Buffered

    // Write:
    //

    // After write:
    //

    // Peek:
    //

    #[inline]
    pub fn write(&mut self, chunk: &[u8]) -> Result<(), TokenizerBailoutReason> {
        let chunk_len = chunk.len();

        if self.watermark + chunk_len <= self.capacity {
            let new_watermark = self.watermark + chunk_len;

            (&mut self.bytes[self.watermark..new_watermark]).copy_from_slice(&chunk);
            self.watermark = new_watermark;

            Ok(())
        } else {
            Err(TokenizerBailoutReason::BufferCapacityExceeded)
        }
    }

    #[inline]
    pub fn peek_at(&self, pos: usize) -> Option<u8> {
        if pos < self.watermark {
            Some(self.bytes[pos])
        } else {
            None
        }
    }
}

impl Deref for InputChunk {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        &self.bytes[..self.watermark]
    }
}

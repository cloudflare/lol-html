use std::ops::Deref;
use tokenizer::TokenizerBailoutReason;

pub struct Buffer {
    bytes: Box<[u8]>,
    capacity: usize,
    watermark: usize,
}

impl Buffer {
    pub fn new(capacity: usize) -> Self {
        Buffer {
            bytes: vec![0; capacity].into(),
            capacity,
            watermark: 0,
        }
    }

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

    // Rename buffer to input
    // 1. Set chunk as initial bytes
    // 2. If current token still exists after parsing then:
    //    b.Move data from `bytes` start at raw_start up to the end into `buffer`
    //    a.Use buffer as `bytes` if it's not currently `bytes`
    // 3. If after parsing whole input has been consumed then set
    // switch back to using chunk as `bytes`

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

impl Deref for Buffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        &self.bytes[..self.watermark]
    }
}

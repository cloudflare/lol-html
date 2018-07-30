mod token;

#[macro_use]
mod state_machine_dsl;

#[macro_use]
mod syntax;

pub use self::token::{Attribute, Token, RawSubslice};

const DEFAULT_ATTR_BUFFER_CAPACITY: usize = 256;

#[derive(Debug)]
pub struct BufferCapacityExceededError<'c> {
    unprocessed_buffer: &'c [u8],
}

// 2. Implement simple states via function calls
// 3. Define macroses and split into syntax files
// 4. Add precommit hook
// 4. Tag name hash
// 5. Implement raw
// 6. Implement streaming
// 7. Implement in-state loops
// 8. Implement re-looper like state embedding
// 9. Implement buffer capacity error recovery (?)
// 10. Parse errors

pub struct Tokenizer<'t, H: FnMut(Token, Option<&[u8]>)> {
    buffer: Box<[u8]>,
    buffer_capacity: usize,
    buffer_watermark: usize,
    pos: usize,
    raw_start: usize,
    finished: bool,
    state_enter: bool,
    token_handler: H,
    state: fn(&mut Tokenizer<'t, H>, Option<u8>),
    current_token: Token<'t>,
    attr_buffer: Vec<Attribute>,
}

define_state_machine!();

impl<'t, H: FnMut(Token, Option<&[u8]>)> Tokenizer<'t, H> {
    pub fn new(buffer_capacity: usize, token_handler: H) -> Tokenizer<'t, H> {
        Tokenizer {
            buffer: vec![0; buffer_capacity].into(),
            buffer_capacity,
            buffer_watermark: 0,
            pos: 0,
            raw_start: 0,
            finished: false,
            state_enter: true,
            token_handler,
            state: Tokenizer::data_state,
            current_token: Token::Eof,
            attr_buffer: Vec::with_capacity(DEFAULT_ATTR_BUFFER_CAPACITY)
        }
    }

    pub fn write(&mut self, chunk: Vec<u8>) -> Result<(), BufferCapacityExceededError> {
        let chunk_len = chunk.len();

        if self.buffer_watermark + chunk_len > self.buffer_capacity {
            return Err(BufferCapacityExceededError {
                unprocessed_buffer: &self.buffer[0..self.buffer_watermark],
            });
        }

        let new_watermark = self.buffer_watermark + chunk_len;

        (&mut self.buffer[self.buffer_watermark..new_watermark]).copy_from_slice(&chunk);
        self.buffer_watermark = new_watermark;

        while !self.finished {
            let ch = if self.pos < self.buffer_watermark {
                Some(self.buffer[self.pos])
            } else {
                None
            };

            (self.state)(self, ch);

            self.pos += 1;
        }

        Ok(())
    }

    pub fn set_state(&mut self, state: fn(&mut Tokenizer<'t, H>, Option<u8>)) {
        self.state = state;
    }
}

mod token;

pub use self::token::Token;

#[derive(Debug)]
pub struct BufferCapacityExceededError<'c> {
    unprocessed_buffer: &'c [u8],
}

// 2. Implement simple states via function calls
// 3. Define macroses and split into syntax files
// 4. Tag name hash
// 5. Implement raw
// 6. Implement streaming
// 7. Implement in-state loops
// 8. Implement re-looper like state embedding
// 9. Implement buffer capacity error recovery (?)
// 10. Parse errors

macro_rules! action {
    (emit_eof ~ $t:ident) => (
        ($t.token_handler)(&Token::Eof);
        $t.finished = true;
    );
}

// TODO: pub vs private
macro_rules! states {
    ($($name: ident { $($actions:tt)* })*) => {
        impl<'t, H: FnMut(&Token)> Tokenizer<'t, H> {
           $(pub fn $name(&mut self, ch: Option<u8>) {
               // NOTE: rust compiler is unhappy about `self` being passed
               // as an identifier token, so to trick it we just declare a
               // local variable that serves as an alias.
               let t = self;
               action!($($actions)* ~ t);
           })*
        }
    };
}

pub struct Tokenizer<'t, H: FnMut(&Token)> {
    buffer: Box<[u8]>,
    buffer_capacity: usize,
    buffer_watermark: usize,
    pos: usize,
    finished: bool,
    state_enter: bool,
    token_handler: H,
    state: fn(&mut Tokenizer<'t, H>, Option<u8>),
}

impl<'t, H: FnMut(&Token)> Tokenizer<'t, H> {
    pub fn new(buffer_capacity: usize, token_handler: H) -> Tokenizer<'t, H> {
        Tokenizer {
            buffer: vec![0; buffer_capacity].into_boxed_slice(),
            buffer_capacity,
            buffer_watermark: 0,
            pos: 0,
            finished: false,
            state_enter: true,
            token_handler,
            state: Tokenizer::data_state,
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
            self.pos += 1;

            let ch = if self.pos < self.buffer_watermark {
                Some(self.buffer[self.pos])
            } else {
                None
            };

            (self.state)(self, ch);
        }

        Ok(())
    }

    pub fn set_state(&mut self, state: fn(&mut Tokenizer<'t, H>, Option<u8>)) {
        self.state = state;
    }
}

states!(
    data_state {
        emit_eof
    }

    plain_text_state {
        emit_eof
    }

    rcdata_state {
        emit_eof
    }

    raw_text_state {
        emit_eof
    }

    script_data_state {
        emit_eof
    }

    cdata_section_state {
        emit_eof
    }
);

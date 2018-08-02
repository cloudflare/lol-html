mod buffer;
mod lex_result;

#[macro_use]
mod state_machine_dsl;

#[macro_use]
mod syntax;

use self::buffer::Buffer;
pub use self::lex_result::*;
use std::cell::RefCell;
use std::rc::Rc;

const DEFAULT_ATTR_BUFFER_CAPACITY: usize = 256;

// 4. Add precommit hook
// 4. Tag name hash
// 5. Implement raw
// 6. Implement streaming
// 7. Implement in-state loops
// 8. Enable LTO
// 9. Implement re-looper like state embedding
// 10. Implement buffer capacity error recovery (?)
// 11. Parse errors
// 12. Attr buffer limits?
// 13. Range slice for raw?

pub struct Tokenizer<'t, H: FnMut(LexResult)> {
    buffer: Buffer,
    pos: usize,
    raw_start: usize,
    token_part_start: usize,
    finished: bool,
    state_enter: bool,
    token_handler: H,
    state: fn(&mut Tokenizer<'t, H>, Option<u8>),
    current_token: Option<ShallowToken>,
    current_attr: Option<ShallowAttribute>,
    closing_quote: u8,
    attr_buffer: Rc<RefCell<Vec<ShallowAttribute>>>,
}

define_state_machine!();

impl<'t, H: FnMut(LexResult)> Tokenizer<'t, H> {
    pub fn new(buffer_capacity: usize, token_handler: H) -> Tokenizer<'t, H> {
        Tokenizer {
            buffer: Buffer::new(buffer_capacity),
            pos: 0,
            raw_start: 0,
            token_part_start: 0,
            finished: false,
            state_enter: true,
            token_handler,
            state: Tokenizer::data_state,
            current_token: None,
            current_attr: None,
            closing_quote: b'"',
            attr_buffer: Rc::new(RefCell::new(Vec::with_capacity(
                DEFAULT_ATTR_BUFFER_CAPACITY,
            ))),
        }
    }

    pub fn write(&mut self, chunk: Vec<u8>) -> Result<(), &'static str> {
        self.buffer.write(chunk)?;

        while !self.finished {
            let ch = self.buffer.peek_at(self.pos);

            (self.state)(self, ch);

            self.pos += 1;
        }

        Ok(())
    }

    pub fn set_state(&mut self, state: fn(&mut Tokenizer<'t, H>, Option<u8>)) {
        self.state = state;
    }
}

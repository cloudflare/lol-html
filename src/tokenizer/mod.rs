mod buffer;
mod lex_result;
mod tag_name_hash;

#[macro_use]
mod state_machine_dsl;

#[macro_use]
mod syntax;

#[macro_use]
mod testing_api;

use self::buffer::Buffer;
pub use self::lex_result::*;
pub use self::tag_name_hash::*;
pub use self::testing_api::*;
use std::cell::RefCell;
use std::rc::Rc;

const DEFAULT_ATTR_BUFFER_CAPACITY: usize = 256;

// About feedback:
// We need to maintain "non-lexical" tree only for tags
// tags that can affect text parsing modes.
// For that we need to analyze tree construction modes
// and find those cases where start tags that initiate
// text parsing modes can be ignored. Then we need to implement
// same old feedback simulation, but also simulate limited
// subset of insertion modes to know when to ignore particular
// start tag that initiates text parsing.

// 2. Use single implementation of state from testing API,
// for deserialization use strings (update example to use testing API
// and create script for trace that enables all required features)
// 3. Enable feedback tests
// 4. Implement simple feedback to not be blocked on it

// 6. Implement feedback
// 6. Don't emit character immidiately, extend existing
// 6. Implement streaming
// 7. Implement in-state loops
// 8. Enable LTO
// 9. Implement re-looper like state embedding
// 10. Implement buffer capacity error recovery (?)
// 11. Parse errors
// 12. Attr buffer limits?
// 13. Range slice for raw?

pub struct Tokenizer<'t, TokenHandler: FnMut(LexResult)> {
    buffer: Buffer,
    pos: usize,
    raw_start: usize,
    token_part_start: usize,
    cdata_end_pos: usize,
    finished: bool,
    state_enter: bool,
    token_handler: TokenHandler,
    state: fn(&mut Tokenizer<'t, TokenHandler>, Option<u8>),
    current_token: Option<ShallowToken>,
    current_attr: Option<ShallowAttribute>,
    last_start_tag_name_hash: Option<u64>,
    closing_quote: u8,
    attr_buffer: Rc<RefCell<Vec<ShallowAttribute>>>,

    #[cfg(feature = "testing_api")]
    text_parsing_mode_change_handler: Option<&'t mut FnMut(TextParsingMode)>,
}

define_state_machine!();

impl<'t, TokenHandler: FnMut(LexResult)> Tokenizer<'t, TokenHandler> {
    pub fn new(buffer_capacity: usize, token_handler: TokenHandler) -> Self {
        Tokenizer {
            buffer: Buffer::new(buffer_capacity),
            pos: 0,
            raw_start: 0,
            token_part_start: 0,
            cdata_end_pos: 0,
            finished: false,
            state_enter: true,
            token_handler,
            state: Tokenizer::data_state,
            current_token: None,
            current_attr: None,
            last_start_tag_name_hash: None,
            closing_quote: b'"',
            attr_buffer: Rc::new(RefCell::new(Vec::with_capacity(
                DEFAULT_ATTR_BUFFER_CAPACITY,
            ))),

            #[cfg(feature = "testing_api")]
            text_parsing_mode_change_handler: None,
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

    #[cfg(feature = "testing_api")]
    pub fn set_state(&mut self, state: fn(&mut Tokenizer<'t, TokenHandler>, Option<u8>)) {
        self.state = state;
    }

    #[cfg(feature = "testing_api")]
    pub fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
        self.last_start_tag_name_hash = name_hash;
    }

    #[cfg(feature = "testing_api")]
    pub fn set_text_parsing_mode_change_handler(
        &mut self,
        handler: &'t mut FnMut(TextParsingMode),
    ) {
        self.text_parsing_mode_change_handler = Some(handler);
    }
}

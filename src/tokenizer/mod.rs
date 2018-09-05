mod buffer;

#[macro_use]
mod state_machine_dsl;

#[macro_use]
mod syntax;

#[cfg(feature = "testing_api")]
#[macro_use]
mod text_parsing_mode;

use self::buffer::Buffer;
use lex_unit::handler::*;
use lex_unit::*;
use std::cell::RefCell;
use std::rc::Rc;
use tag_name_hash::*;

#[cfg(feature = "testing_api")]
pub use self::text_parsing_mode::*;

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

// OPTIMISATION IDEA:
// Instead of using direct token flow approach when tokens come
// from tokenizer through a number of handlers to serilizer
// use different approach: connect tokenizer directly with
// serializer, so in most cases serializer directly writes
// original chunk to output, and only if handler rewrites
// something - rewrite only specified subchunk of the original chunk.
// Luckily, it's is easy to do considering that we use ranges
// instead of real pointers.

// OPTIMISATION IDEA:
// Skip parsing particular parts of tokens if we are not interested in them:
// e.g. for tree builder simulator we don't need anything besides start and end
// tag names, so we can avoid collecting token attributes. If we have only tag
// selectors we can avoid collecting attributes as well.

// OPTIMISATION IDEA:
// We can avoid using tokens for state adjustment: just introduce separate
// events for tag name parsing and different type of handler trait. So,
// we will not invoke simulator handler for text or doctype.

// OPTIMISATION IDEA:
// All selectors are based on start tag, so unless we have a matching
// start tag, we can avoid capturing and producing any tokens.

// OPTIMISATION IDEA:
// Have two parsers: eager and full, both generated from the same syntax definition,
// but having different action definitions. Eager parser doesn't produce tokens, it
// just notifies matcher that it have seen particular start tag (all matching is based on tags).
// If matcher says that tag matches, we switch to the full parser that actually produce
// tokens. Then action executor tells if we should replace token. We run token through
// serializer and substitute it into original chunk. Parser will share tree builder simulator.

// 1. Add benchmark
// 2. Implement simple feedback to not be blocked on it

// 6. Implement feedback
// 7. Move lex result out of tokenizer, use it to store information
// for the whole pipeline: such as namespace and if it ignored by tree builder
// 5. Make all buffer size adjustable, propagate capacity errors to write function
// 6. Don't emit character immidiately, extend existing
// 6. Implement streaming
// 7. Implement in-state loops
// 8. Enable LTO
// 9. Implement re-looper like state embedding
// 10. Implement buffer capacity error recovery (?)
// 11. Parse errors
// 12. Attr buffer limits?
// 13. Range slice for raw?

pub struct Tokenizer<'t, H> {
    buffer: Buffer,
    pos: usize,
    raw_start: usize,
    token_part_start: usize,
    cdata_end_pos: usize,
    finished: bool,
    state_enter: bool,
    allow_cdata: bool,
    lex_res_handler: H,
    state: fn(&mut Tokenizer<'t, H>, Option<u8>),
    current_token: Option<ShallowToken>,
    current_attr: Option<ShallowAttribute>,
    last_start_tag_name_hash: Option<u64>,
    closing_quote: u8,
    attr_buffer: Rc<RefCell<Vec<ShallowAttribute>>>,

    #[cfg(feature = "testing_api")]
    text_parsing_mode_change_handler: Option<&'t mut FnMut(TextParsingMode)>,
}

define_state_machine!();

impl<'t, H: LexUnitHandlerWithFeedback> Tokenizer<'t, H> {
    pub fn new(buffer_capacity: usize, lex_res_handler: H) -> Self {
        Tokenizer {
            buffer: Buffer::new(buffer_capacity),
            pos: 0,
            raw_start: 0,
            token_part_start: 0,
            cdata_end_pos: 0,
            finished: false,
            state_enter: true,
            allow_cdata: false,
            lex_res_handler,
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
    pub fn set_state(&mut self, state: fn(&mut Tokenizer<'t, H>, Option<u8>)) {
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

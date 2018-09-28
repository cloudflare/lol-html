#[macro_use]
mod state_machine_dsl;

#[macro_use]
mod syntax;

mod buffer;
mod tree_builder_simulator;

use self::buffer::Buffer;
use self::tree_builder_simulator::*;
use lex_unit::handler::*;
use lex_unit::*;
use std::cell::RefCell;
use std::rc::Rc;
use tag_name::TagName;

#[cfg(feature = "testing_api")]
pub use self::tree_builder_simulator::{TextParsingMode, TextParsingModeSnapshot};

const DEFAULT_ATTR_BUFFER_CAPACITY: usize = 256;

// 1. Add benchmark
// 6. Implement feedback
// 5. Make all buffer sizes adjustable, propagate capacity errors to write function
// 6. Don't emit character immidiately, extend existing
// 6. Implement streaming
// 7. Implement in-state loops
// 8. Enable LTO
// 9. Implement re-looper like state embedding
// 12. Attr buffer limits?

#[derive(Debug, Copy, Clone)]
pub enum TokenizerErrorKind {
    BufferCapacityExceeded,
    TextParsingAmbiguity,
}

#[cfg(feature = "testing_api")]
impl<'s> From<&'s str> for TokenizerErrorKind {
    fn from(kind: &'s str) -> Self {
        match kind {
            "BufferCapacityExceeded" => TokenizerErrorKind::BufferCapacityExceeded,
            "TextParsingAmbiguity" => TokenizerErrorKind::TextParsingAmbiguity,
            _ => panic!("Unknown tokenizer error kind"),
        }
    }
}

pub type TokenizerState<'t, H> =
    fn(&mut Tokenizer<'t, H>, Option<u8>) -> Result<(), TokenizerErrorKind>;

pub struct Tokenizer<'t, H> {
    buffer: Buffer,
    pos: usize,
    raw_start: usize,
    token_part_start: usize,
    finished: bool,
    state_enter: bool,
    allow_cdata: bool,
    lex_unit_handler: H,
    state: TokenizerState<'t, H>,
    current_token: Option<ShallowToken>,
    current_attr: Option<ShallowAttribute>,
    last_start_tag_name_hash: Option<u64>,
    closing_quote: u8,
    attr_buffer: Rc<RefCell<Vec<ShallowAttribute>>>,
    tree_builder_simulator: TreeBuilderSimulator,

    #[cfg(feature = "testing_api")]
    text_parsing_mode_change_handler: Option<&'t mut TextParsingModeChangeHandler>,
}

define_state_machine!();

impl<'t, H: LexUnitHandler> Tokenizer<'t, H> {
    pub fn new(buffer_capacity: usize, lex_unit_handler: H) -> Self {
        Tokenizer {
            buffer: Buffer::new(buffer_capacity),
            pos: 0,
            raw_start: 0,
            token_part_start: 0,
            finished: false,
            state_enter: true,
            allow_cdata: false,
            lex_unit_handler,
            state: Tokenizer::data_state,
            current_token: None,
            current_attr: None,
            last_start_tag_name_hash: None,
            closing_quote: b'"',
            attr_buffer: Rc::new(RefCell::new(Vec::with_capacity(
                DEFAULT_ATTR_BUFFER_CAPACITY,
            ))),
            tree_builder_simulator: TreeBuilderSimulator::default(),

            #[cfg(feature = "testing_api")]
            text_parsing_mode_change_handler: None,
        }
    }

    pub fn write(&mut self, chunk: &[u8]) -> Result<(), TokenizerErrorKind> {
        self.buffer.write(chunk)?;

        while !self.finished {
            let ch = self.buffer.peek_at(self.pos);

            (self.state)(self, ch)?;

            self.pos += 1;
        }

        Ok(())
    }

    #[cfg(feature = "testing_api")]
    pub fn set_state(&mut self, state: TokenizerState<'t, H>) {
        self.state = state;
    }

    #[cfg(feature = "testing_api")]
    pub fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
        self.last_start_tag_name_hash = name_hash;
    }

    #[cfg(feature = "testing_api")]
    pub fn set_text_parsing_mode_change_handler(
        &mut self,
        handler: &'t mut TextParsingModeChangeHandler,
    ) {
        self.text_parsing_mode_change_handler = Some(handler);
    }
}

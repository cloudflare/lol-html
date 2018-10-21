#[macro_use]
mod state_machine_dsl;

#[macro_use]
mod syntax;

#[macro_use]
mod tag_name;

mod lex_unit;
mod token;
mod tree_builder_simulator;

pub use self::lex_unit::LexUnit;
pub use self::tag_name::TagName;
pub use self::token::*;
use self::tree_builder_simulator::*;
use base::{Bytes, Range};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "testing_api")]
pub use self::tree_builder_simulator::{TextParsingMode, TextParsingModeSnapshot};

const DEFAULT_ATTR_BUFFER_CAPACITY: usize = 256;

pub trait LexUnitHandler {
    fn handle(&mut self, lex_unit: &LexUnit);
}

#[cfg(feature = "testing_api")]
impl<F: FnMut(&LexUnit)> LexUnitHandler for F {
    fn handle(&mut self, lex_unit: &LexUnit) {
        self(lex_unit);
    }
}

pub enum ParsingLoopDirective {
    Break,
    Continue,
}

#[derive(Debug, Copy, Clone)]
pub enum TokenizerBailoutReason {
    BufferCapacityExceeded,
    TextParsingAmbiguity,
    MaxTagNestingReached,
}

pub type TokenizerState<H> = fn(&mut Tokenizer<H>, &Bytes, Option<&u8>)
    -> Result<ParsingLoopDirective, TokenizerBailoutReason>;

pub struct Tokenizer<H> {
    pos: usize,
    lex_unit_start: usize,
    token_part_start: usize,
    last_chunk: bool,
    state_enter: bool,
    allow_cdata: bool,
    lex_unit_handler: H,
    state: TokenizerState<H>,
    current_token: Option<TokenView>,
    current_attr: Option<AttributeView>,
    last_start_tag_name_hash: Option<u64>,
    closing_quote: u8,
    attr_buffer: Rc<RefCell<Vec<AttributeView>>>,
    tree_builder_simulator: TreeBuilderSimulator,

    #[cfg(feature = "testing_api")]
    text_parsing_mode_change_handler: Option<Box<TextParsingModeChangeHandler>>,
}

define_state_machine!();

impl<H: LexUnitHandler> Tokenizer<H> {
    pub fn new(lex_unit_handler: H) -> Self {
        Tokenizer {
            pos: 0,
            lex_unit_start: 0,
            token_part_start: 0,
            last_chunk: false,
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

    pub fn tokenize_chunk(
        &mut self,
        input_chunk: &Bytes,
        last_chunk: bool,
    ) -> Result<usize, TokenizerBailoutReason> {
        self.last_chunk = last_chunk;

        loop {
            let ch = input_chunk.get(self.pos);
            let directive = (self.state)(self, &input_chunk, ch)?;

            self.pos += 1;

            if let ParsingLoopDirective::Break = directive {
                break;
            }
        }

        Ok(self.lex_unit_start)
    }

    #[cfg(feature = "testing_api")]
    pub fn set_state(&mut self, state: TokenizerState<H>) {
        self.state = state;
    }

    #[cfg(feature = "testing_api")]
    pub fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
        self.last_start_tag_name_hash = name_hash;
    }

    #[cfg(feature = "testing_api")]
    pub fn set_text_parsing_mode_change_handler(
        &mut self,
        handler: Box<TextParsingModeChangeHandler>,
    ) {
        self.text_parsing_mode_change_handler = Some(handler);
    }
}

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
use base::{Align, Chunk, Range};
use errors::Error;
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

pub type TokenizerState<H> = fn(&mut Tokenizer<H>, &Chunk) -> Result<ParsingLoopDirective, Error>;

pub struct Tokenizer<H> {
    next_pos: usize,
    lex_unit_start: usize,
    token_part_start: usize,
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
    text_parsing_mode_change_handler: Option<Box<dyn TextParsingModeChangeHandler>>,
}

define_state_machine!();

impl<H: LexUnitHandler> Tokenizer<H> {
    pub fn new(lex_unit_handler: H) -> Self {
        Tokenizer {
            next_pos: 0,
            lex_unit_start: 0,
            token_part_start: 0,
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

    pub fn tokenize(&mut self, input: &Chunk) -> Result<usize, Error> {
        loop {
            let directive = (self.state)(self, input)?;

            if let ParsingLoopDirective::Break = directive {
                break;
            }
        }

        let blocked_byte_count = input.len() - self.lex_unit_start;

        if !input.is_last() {
            self.adjust_for_next_input();
        }

        Ok(blocked_byte_count)
    }

    fn adjust_for_next_input(&mut self) {
        let offset = self.lex_unit_start;

        self.lex_unit_start = 0;

        self.next_pos.align(offset + 1);
        self.token_part_start.align(offset);
        self.current_token.align(offset);
        self.current_attr.align(offset);
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
        handler: Box<dyn TextParsingModeChangeHandler>,
    ) {
        self.text_parsing_mode_change_handler = Some(handler);
    }
}

#[macro_use]
mod actions;
mod conditions;
mod lex_unit;
mod token;

pub use self::lex_unit::*;
pub use self::token::*;
use base::{Align, Chunk, Cursor, Range};
use errors::Error;
use std::cell::RefCell;
use std::rc::Rc;
pub use tokenizer::tag_name::TagName;
use tokenizer::tree_builder_simulator::*;

const DEFAULT_ATTR_BUFFER_CAPACITY: usize = 256;

pub enum ParsingLoopDirective {
    Break,
    Continue,
}

pub type TokenizerState<H> = fn(&mut Tokenizer<H>, &Chunk) -> Result<ParsingLoopDirective, Error>;

pub struct Tokenizer<H: LexUnitHandler> {
    input_cursor: Cursor,
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

impl<H: LexUnitHandler> Tokenizer<H> {
    define_state_machine!();

    pub fn new(lex_unit_handler: H) -> Self {
        Tokenizer {
            input_cursor: Cursor::default(),
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
            self.adjust_for_next_input()
        }

        Ok(blocked_byte_count)
    }

    fn adjust_for_next_input(&mut self) {
        self.input_cursor.align(self.lex_unit_start);
        self.token_part_start.align(self.lex_unit_start);
        self.current_token.align(self.lex_unit_start);
        self.current_attr.align(self.lex_unit_start);

        self.lex_unit_start = 0;
    }

    #[inline]
    pub fn set_text_parsing_mode(&mut self, mode: TextParsingMode) {
        self.switch_state(match mode {
            TextParsingMode::Data => Tokenizer::data_state,
            TextParsingMode::PlainText => Tokenizer::plaintext_state,
            TextParsingMode::RCData => Tokenizer::rcdata_state,
            TextParsingMode::RawText => Tokenizer::rawtext_state,
            TextParsingMode::ScriptData => Tokenizer::script_data_state,
            TextParsingMode::CDataSection => Tokenizer::cdata_section_state,
        });
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

    fn handle_tree_builder_feedback(
        &mut self,
        feedback: TreeBuilderFeedback,
        lex_unit: &LexUnit,
    ) -> Option<ParsingLoopDirective> {
        let mut feedback = feedback;

        loop {
            match feedback {
                TreeBuilderFeedback::Adjust(adjustment) => {
                    return self.apply_adjustment(adjustment);
                }
                TreeBuilderFeedback::RequestStartTagToken(reason) => {
                    let token = lex_unit
                        .get_token()
                        .expect("There should be a token at this point");

                    feedback = self
                        .tree_builder_simulator
                        .fulfill_start_tag_token_request(&token, reason);
                }
                TreeBuilderFeedback::RequestEndTagToken => {
                    let token = lex_unit
                        .get_token()
                        .expect("There should be a token at this point");

                    feedback = self
                        .tree_builder_simulator
                        .fulfill_end_tag_token_request(&token);
                }
                TreeBuilderFeedback::RequestSelfClosingFlag => match lex_unit.get_token_view() {
                    Some(&TokenView::StartTag { self_closing, .. }) => {
                        feedback = self
                            .tree_builder_simulator
                            .fulfill_self_closing_flag_request(self_closing);
                    }
                    _ => unreachable!("Token should be a start tag at this point"),
                },
                TreeBuilderFeedback::None => break,
            }
        }

        None
    }

    fn apply_adjustment(
        &mut self,
        adjustment: TokenizerAdjustment,
    ) -> Option<ParsingLoopDirective> {
        match adjustment {
            TokenizerAdjustment::SwitchTextParsingMode(mode) => {
                notify_text_parsing_mode_change!(self, mode);
                self.set_text_parsing_mode(mode);
                Some(ParsingLoopDirective::Continue)
            }
            TokenizerAdjustment::SetAllowCdata(allow_cdata) => {
                self.allow_cdata = allow_cdata;
                None
            }
        }
    }

    #[inline]
    fn switch_state(&mut self, state: TokenizerState<H>) {
        self.state = state;
        self.state_enter = true;
    }

    #[inline]
    fn emit_lex_unit<'c>(
        &mut self,
        input: &'c Chunk,
        token: Option<TokenView>,
        raw_range: Option<Range>,
    ) -> LexUnit<'c> {
        let lex_unit = LexUnit::new(input, token, raw_range);

        self.lex_unit_handler.handle(&lex_unit);

        lex_unit
    }

    #[inline]
    fn emit_lex_unit_with_raw<'c>(
        &mut self,
        input: &'c Chunk,
        token: Option<TokenView>,
        raw_end: usize,
    ) -> LexUnit<'c> {
        let raw_range = Some(Range {
            start: self.lex_unit_start,
            end: raw_end,
        });

        self.lex_unit_start = raw_end;

        self.emit_lex_unit(input, token, raw_range)
    }

    #[inline]
    fn emit_lex_unit_with_raw_inclusive<'c>(
        &mut self,
        input: &'c Chunk,
        token: Option<TokenView>,
    ) -> LexUnit<'c> {
        let raw_end = self.input_cursor.pos() + 1;

        self.emit_lex_unit_with_raw(input, token, raw_end)
    }

    #[inline]
    fn emit_lex_unit_with_raw_exclusive<'c>(
        &mut self,
        input: &'c Chunk,
        token: Option<TokenView>,
    ) -> LexUnit<'c> {
        let raw_end = self.input_cursor.pos();

        self.emit_lex_unit_with_raw(input, token, raw_end)
    }
}

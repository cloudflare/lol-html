#[macro_use]
mod tag_name;

#[macro_use]
mod state_machine;

mod outputs;
mod text_parsing_mode;
mod tree_builder_simulator;

use self::state_machine::{
    EagerStateMachine, FullStateMachine, ParsingLoopTerminationReason, StateMachine,
};
use self::tree_builder_simulator::TreeBuilderSimulator;
use base::Chunk;
use crate::Error;
use std::cell::RefCell;
use std::rc::Rc;

pub use self::outputs::*;
pub use self::tag_name::TagName;
pub use self::text_parsing_mode::*;

pub enum NextOutputType {
    TagPreview,
    LexUnit,
}

declare_handler! {
    LexUnitHandler(&LexUnit)
}

declare_handler! {
    TagLexUnitHandler(&LexUnit) -> NextOutputType
}

declare_handler! {
    TagPreviewHandler(&TagPreview) -> NextOutputType
}

pub struct Tokenizer<LH, TH, PH>
where
    LH: LexUnitHandler,
    TH: TagLexUnitHandler,
    PH: TagPreviewHandler,
{
    full_sm: FullStateMachine<LH, TH>,
    eager_sm: EagerStateMachine<PH>,
    next_output_type: NextOutputType,
}

// NOTE: dynamic dispatch can't be used for the StateMachine trait
// because it's not object-safe due to the usage of `Self` in function
// signatures, so we use this macro instead.
macro_rules! with_current_sm {
    ($self:tt, { sm.$fn:ident($($args:tt)*) }) => {
        match $self.next_output_type {
            NextOutputType::TagPreview => $self.eager_sm.$fn($($args)*),
            NextOutputType::LexUnit => $self.full_sm.$fn($($args)*),
        }
    };
}

impl<LH, TH, PH> Tokenizer<LH, TH, PH>
where
    LH: LexUnitHandler,
    TH: TagLexUnitHandler,
    PH: TagPreviewHandler,
{
    pub fn new(lex_unit_handler: LH, tag_lex_unit_handler: TH, tag_preview_handler: PH) -> Self {
        let tree_builder_simulator = Rc::new(RefCell::new(TreeBuilderSimulator::default()));

        Tokenizer {
            full_sm: FullStateMachine::new(
                lex_unit_handler,
                tag_lex_unit_handler,
                &tree_builder_simulator,
            ),
            eager_sm: EagerStateMachine::new(tag_preview_handler, &tree_builder_simulator),
            next_output_type: NextOutputType::TagPreview,
        }
    }

    pub fn tokenize(&mut self, chunk: &Chunk) -> Result<usize, Error> {
        let mut loop_termination_reason = with_current_sm!(self, { sm.run_parsing_loop(chunk) })?;

        loop {
            match loop_termination_reason {
                ParsingLoopTerminationReason::OutputTypeSwitch {
                    next_type,
                    sm_bookmark,
                } => {
                    self.next_output_type = next_type;

                    loop_termination_reason =
                        with_current_sm!(self, { sm.continue_from_bookmark(chunk, &sm_bookmark) })?;
                }
                ParsingLoopTerminationReason::EndOfInput { blocked_byte_count } => {
                    return Ok(blocked_byte_count)
                }
            }
        }
    }
}

#[cfg(feature = "testing_api")]
impl<LH, TH, PH> Tokenizer<LH, TH, PH>
where
    LH: LexUnitHandler,
    TH: TagLexUnitHandler,
    PH: TagPreviewHandler,
{
    pub fn set_next_output_type(&mut self, ty: NextOutputType) {
        self.next_output_type = ty;
    }

    pub fn switch_text_parsing_mode(&mut self, mode: TextParsingMode) {
        with_current_sm!(self, { sm.switch_text_parsing_mode(mode) });
    }

    pub fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
        with_current_sm!(self, { sm.set_last_start_tag_name_hash(name_hash) });
    }

    pub fn get_full_sm(&mut self) -> &mut FullStateMachine<LH, TH> {
        &mut self.full_sm
    }
}

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

    #[inline]
    pub fn tokenize(&mut self, chunk: &Chunk) -> Result<usize, Error> {
        match with_current_sm!(self, { sm.run_parsing_loop(chunk) })? {
            ParsingLoopTerminationReason::EndOfInput { blocked_byte_count } => {
                Ok(blocked_byte_count)
            }
            ParsingLoopTerminationReason::OutputTypeSwitch(_) => Ok(0),
        }
    }

    #[cfg(feature = "testing_api")]
    pub fn set_next_output_type(&mut self, ty: NextOutputType) {
        self.next_output_type = ty;
    }

    #[cfg(feature = "testing_api")]
    pub fn set_text_parsing_mode(&mut self, mode: TextParsingMode) {
        with_current_sm!(self, { sm.set_text_parsing_mode(mode) });
    }

    #[cfg(feature = "testing_api")]
    pub fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
        with_current_sm!(self, { sm.set_last_start_tag_name_hash(name_hash) });
    }

    #[cfg(feature = "testing_api")]
    pub fn set_text_parsing_mode_change_handler(
        &mut self,
        handler: Box<dyn TextParsingModeChangeHandler>,
    ) {
        with_current_sm!(self, { sm.set_text_parsing_mode_change_handler(handler) });
    }
}

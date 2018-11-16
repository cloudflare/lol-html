#[macro_use]
mod tag_name;

#[macro_use]
mod state_machine;

mod outputs;
mod state_machine_impls;
mod text_parsing_mode;
mod tree_builder_simulator;

use self::state_machine::*;
use self::state_machine_impls::*;
use self::tree_builder_simulator::TreeBuilderSimulator;
use base::Chunk;
use crate::Error;
use std::cell::RefCell;
use std::rc::Rc;

pub use self::outputs::*;
pub use self::tag_name::TagName;
pub use self::text_parsing_mode::*;

pub enum TagLexUnitResponse {
    SwitchToTagPreviewMode,
    None,
}

pub enum TagPreviewResponse {
    CaptureLexUnits,
    CaptureCurrentTagLexUnitOnly,
    None,
}

pub enum OutputMode {
    LexUnits,
    TagPreviews,
    //NextLexUnitThenTagPreviews
}

declare_handler! {
    LexUnitHandler(&LexUnit)
}

declare_handler! {
    TagLexUnitHandler(&LexUnit) -> TagLexUnitResponse
}

declare_handler! {
    TagPreviewHandler(&TagPreview) -> TagPreviewResponse
}

pub struct Tokenizer<LH, TH, PH>
where
    LH: LexUnitHandler,
    TH: TagLexUnitHandler,
    PH: TagPreviewHandler,
{
    full_sm: FullStateMachine<LH, TH>,
    eager_sm: EagerStateMachine<PH>,
    output_mode: OutputMode,
}

// NOTE: dynamic dispatch can't be used for the StateMachine trait
// because it's not object-safe due to the usage of `Self` in function
// signatures, so we use this macro instead.
#[cfg(feature = "testing_api")]
macro_rules! with_current_sm {
    ($self:tt, { sm.$fn:ident($($args:tt)*) }) => {
        match $self.output_mode {
            OutputMode::TagPreviews => $self.eager_sm.$fn($($args)*),
            OutputMode::LexUnits => $self.full_sm.$fn($($args)*),
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
            output_mode: OutputMode::TagPreviews,
        }
    }

    #[inline]
    pub fn tokenize(&mut self, chunk: &Chunk) -> Result<usize, Error> {
        match self.output_mode {
            OutputMode::LexUnits => {
                let termination_reason = self.full_sm.run_parsing_loop(chunk)?;

                match termination_reason {
                    ParsingLoopTerminationReason::OutputResponse(_) => (),
                    ParsingLoopTerminationReason::EndOfInput { blocked_byte_count } => {
                        return Ok(blocked_byte_count);
                    }
                }
            }
            OutputMode::TagPreviews => {
                let termination_reason = self.eager_sm.run_parsing_loop(chunk)?;

                match termination_reason {
                    ParsingLoopTerminationReason::OutputResponse(_) => (),
                    ParsingLoopTerminationReason::EndOfInput { blocked_byte_count } => {
                        return Ok(blocked_byte_count);
                    }
                }
            }
        }

        Ok(0)
    }

    #[cfg(feature = "testing_api")]
    pub fn set_output_mode(&mut self, mode: OutputMode) {
        self.output_mode = mode;
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

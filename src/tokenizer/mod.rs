#[macro_use]
mod tag_name;

#[macro_use]
mod state_machine;

mod feedback_providers;
mod outputs;
mod text_parsing_mode;

use self::feedback_providers::*;
use self::state_machine::{
    EagerStateMachine, FullStateMachine, ParsingLoopTerminationReason, StateMachine,
};
use base::Chunk;
use crate::Error;
use std::cell::RefCell;
use std::rc::Rc;

pub use self::outputs::*;
pub use self::tag_name::TagName;
pub use self::text_parsing_mode::*;

#[derive(Debug)]
pub enum NextOutputType {
    TagPreview,
    LexUnit,
}

declare_handler! {
    LexUnitHandler(&LexUnit<'_>)
}

declare_handler! {
    TagLexUnitHandler(&LexUnit<'_>) -> NextOutputType
}

declare_handler! {
    TagPreviewHandler(&TagPreview<'_>) -> NextOutputType
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
        let feedback_providers = Rc::new(RefCell::new(FeedbackProviders::default()));

        Tokenizer {
            full_sm: FullStateMachine::new(
                lex_unit_handler,
                tag_lex_unit_handler,
                Rc::clone(&feedback_providers),
            ),
            eager_sm: EagerStateMachine::new(tag_preview_handler, Rc::clone(&feedback_providers)),
            next_output_type: NextOutputType::TagPreview,
        }
    }

    pub fn tokenize(&mut self, input: &Chunk<'_>) -> Result<usize, Error> {
        let mut loop_termination_reason = with_current_sm!(self, { sm.run_parsing_loop(input) })?;

        loop {
            match loop_termination_reason {
                ParsingLoopTerminationReason::OutputTypeSwitch(next_type, sm_bookmark) => {
                    self.next_output_type = next_type;

                    trace!(@continue_from_bookmark sm_bookmark, self.next_output_type, input);

                    loop_termination_reason =
                        with_current_sm!(self, { sm.continue_from_bookmark(input, sm_bookmark) })?;
                }
                ParsingLoopTerminationReason::LexUnitRequiredForAdjustment(sm_bookmark) => {
                    // NOTE: lex unit was required to get tree builder feedback for eager
                    // tokenizer. So we need to spin full state machine and consume lex unit
                    // for the tag, but without emitting it to consumers as they don't expect
                    // lex units at this point.
                    self.next_output_type = NextOutputType::LexUnit;

                    trace!(@continue_from_bookmark sm_bookmark, self.next_output_type, input);

                    loop_termination_reason = self
                        .full_sm
                        .silently_consume_current_tag_only(input, sm_bookmark)?;
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

    pub fn set_tag_confirmation_handler(&mut self, handler: Box<dyn FnMut()>) {
        let handler = Rc::new(RefCell::new(handler));

        self.full_sm.tag_confirmation_handler = Some(Rc::clone(&handler));
        self.eager_sm.tag_confirmation_handler = Some(Rc::clone(&handler));
    }
}

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
use crate::base::Chunk;
use failure::Error;
use std::cell::RefCell;
use std::rc::Rc;

pub use self::outputs::*;
pub use self::state_machine::{LexUnitSink, TagPreviewSink};
pub use self::tag_name::TagName;
pub use self::text_parsing_mode::*;

#[derive(Debug, Copy, Clone)]
pub enum NextOutputType {
    TagPreview,
    LexUnit,
}

impl<S: LexUnitSink> LexUnitSink for Rc<RefCell<S>> {
    #[inline]
    fn handle_tag(&mut self, lex_unit: &LexUnit<'_>) -> NextOutputType {
        self.borrow_mut().handle_tag(lex_unit)
    }

    #[inline]
    fn handle_non_tag_content(&mut self, lex_unit: &LexUnit<'_>) {
        self.borrow_mut().handle_non_tag_content(lex_unit);
    }
}

impl<S: TagPreviewSink> TagPreviewSink for Rc<RefCell<S>> {
    #[inline]
    fn handle_tag_preview(&mut self, tag_preview: &TagPreview<'_>) -> NextOutputType {
        self.borrow_mut().handle_tag_preview(tag_preview)
    }
}

pub trait OutputSink: LexUnitSink + TagPreviewSink {}

pub struct Tokenizer<S: OutputSink> {
    full_sm: FullStateMachine<Rc<RefCell<S>>>,
    eager_sm: EagerStateMachine<Rc<RefCell<S>>>,
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

impl<S: OutputSink> Tokenizer<S> {
    pub fn new(output_sink: &Rc<RefCell<S>>) -> Self {
        let feedback_providers = Rc::new(RefCell::new(FeedbackProviders::default()));

        Tokenizer {
            full_sm: FullStateMachine::new(Rc::clone(output_sink), Rc::clone(&feedback_providers)),
            eager_sm: EagerStateMachine::new(
                Rc::clone(output_sink),
                Rc::clone(&feedback_providers),
            ),
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
                    return Ok(blocked_byte_count);
                }
            }
        }
    }
}

#[cfg(feature = "testing_api")]
impl<S: OutputSink> Tokenizer<S> {
    pub fn set_next_output_type(&mut self, ty: NextOutputType) {
        self.next_output_type = ty;
    }

    pub fn switch_text_parsing_mode(&mut self, mode: TextParsingMode) {
        with_current_sm!(self, { sm.switch_text_parsing_mode(mode) });
    }

    pub fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
        with_current_sm!(self, { sm.set_last_start_tag_name_hash(name_hash) });
    }

    pub fn full_sm(&mut self) -> &mut FullStateMachine<LUH, TLUH> {
        &mut self.full_sm
    }

    pub fn set_tag_confirmation_handler(&mut self, handler: Box<dyn FnMut()>) {
        let handler = Rc::new(RefCell::new(handler));

        self.full_sm.tag_confirmation_handler = Some(Rc::clone(&handler));
        self.eager_sm.tag_confirmation_handler = Some(Rc::clone(&handler));
    }
}

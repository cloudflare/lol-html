#[macro_use]
mod tag_name;

#[macro_use]
mod state_machine;

mod feedback_providers;
mod outputs;
mod text_type;

use self::feedback_providers::*;
use self::state_machine::{
    EagerStateMachine, FullStateMachine, ParsingLoopTerminationReason, StateMachine,
};
use crate::base::Chunk;
use failure::Error;
use std::cell::RefCell;
use std::rc::Rc;

pub use self::outputs::*;
pub use self::state_machine::{LexemeSink, TagPreviewSink};
pub use self::tag_name::TagName;
pub use self::text_type::*;

#[derive(Debug, Copy, Clone)]
pub enum NextOutputType {
    TagPreview,
    Lexeme,
}

impl<S: LexemeSink> LexemeSink for Rc<RefCell<S>> {
    #[inline]
    fn handle_tag(&mut self, lexeme: &Lexeme<'_>) -> NextOutputType {
        self.borrow_mut().handle_tag(lexeme)
    }

    #[inline]
    fn handle_non_tag_content(&mut self, lexeme: &Lexeme<'_>) {
        self.borrow_mut().handle_non_tag_content(lexeme);
    }
}

impl<S: TagPreviewSink> TagPreviewSink for Rc<RefCell<S>> {
    #[inline]
    fn handle_tag_preview(&mut self, tag_preview: &TagPreview<'_>) -> NextOutputType {
        self.borrow_mut().handle_tag_preview(tag_preview)
    }
}

pub trait OutputSink: LexemeSink + TagPreviewSink {}

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
            NextOutputType::Lexeme => $self.full_sm.$fn($($args)*),
        }
    };
}

impl<S: OutputSink> Tokenizer<S> {
    pub fn new(output_sink: &Rc<RefCell<S>>, initial_output_type: NextOutputType) -> Self {
        let feedback_providers = Rc::new(RefCell::new(FeedbackProviders::default()));

        Tokenizer {
            full_sm: FullStateMachine::new(Rc::clone(output_sink), Rc::clone(&feedback_providers)),
            eager_sm: EagerStateMachine::new(
                Rc::clone(output_sink),
                Rc::clone(&feedback_providers),
            ),
            next_output_type: initial_output_type,
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
                ParsingLoopTerminationReason::LexemeRequiredForAdjustment(sm_bookmark) => {
                    // NOTE: lexeme was required to get tree builder feedback for eager
                    // tokenizer. So we need to spin full state machine and consume lexeme
                    // for the tag, but without emitting it to consumers as they don't expect
                    // lexemes at this point.
                    self.next_output_type = NextOutputType::Lexeme;

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
    pub fn switch_text_type(&mut self, text_type: TextType) {
        with_current_sm!(self, { sm.switch_text_type(text_type) });
    }

    pub fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
        with_current_sm!(self, { sm.set_last_start_tag_name_hash(name_hash) });
    }
}

#[macro_use]
mod tag_name;

#[macro_use]
mod state_machine;

mod outputs;
mod state_machine_impls;
mod text_parsing_mode;
mod tree_builder_simulator;

use self::state_machine::*;
use self::state_machine_impls::full::FullStateMachine;
use base::Chunk;
use crate::Error;

pub use self::outputs::*;
pub use self::tag_name::TagName;
pub use self::text_parsing_mode::*;

pub enum ParsingLoopDirective {
    Break,
    Continue,
}

pub struct Tokenizer<S: StateMachine>(S);

impl<S: StateMachine> Tokenizer<S> {
    pub fn tokenize(&mut self, input: &Chunk) -> Result<usize, Error> {
        loop {
            let state = self.0.get_state();
            let directive = state(&mut self.0, input)?;

            if let ParsingLoopDirective::Break = directive {
                break;
            }
        }

        let blocked_byte_count = self.0.get_blocked_byte_count(input);

        if !input.is_last() {
            self.0.adjust_for_next_input()
        }

        Ok(blocked_byte_count)
    }

    #[cfg(feature = "testing_api")]
    pub fn set_text_parsing_mode(&mut self, mode: TextParsingMode) {
        self.0.set_text_parsing_mode(mode);
    }

    #[cfg(feature = "testing_api")]
    pub fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
        self.0.set_last_start_tag_name_hash(name_hash);
    }

    #[cfg(feature = "testing_api")]
    pub fn set_text_parsing_mode_change_handler(
        &mut self,
        handler: Box<dyn TextParsingModeChangeHandler>,
    ) {
        self.0.set_text_parsing_mode_change_handler(handler);
    }
}

pub type FullTokenizer<H> = Tokenizer<FullStateMachine<H>>;

impl<H: LexUnitHandler> FullTokenizer<H> {
    pub fn new(lex_unit_handler: H) -> Self {
        Tokenizer(FullStateMachine::new(lex_unit_handler))
    }
}

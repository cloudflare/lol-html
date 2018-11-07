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
use base::Chunk;
use crate::Error;

pub use self::outputs::*;
pub use self::tag_name::TagName;
pub use self::text_parsing_mode::*;

// NOTE: dynamic dispatch can't be used for the StateMachine trait
// because it uses `Self`, so we use this macro instead.
macro_rules! with_current_sm {
    ($self:tt, { sm.$fn:ident($($args:tt)*) }) => {
        if $self.tag_preview_mode {
            $self.eager_sm.$fn($($args)*)
        } else {
            $self.full_sm.$fn($($args)*)
        }
    };
}

pub struct Tokenizer<LH, TH>
where
    LH: LexUnitHandler,
    TH: TagPreviewHandler,
{
    full_sm: FullStateMachine<LH>,
    eager_sm: EagerStateMachine<TH>,
    tag_preview_mode: bool,
}

impl<LH, TH> Tokenizer<LH, TH>
where
    LH: LexUnitHandler,
    TH: TagPreviewHandler,
{
    pub fn new(lex_unit_handler: LH, tag_preview_handler: TH) -> Self {
        Tokenizer {
            full_sm: FullStateMachine::new(lex_unit_handler),
            eager_sm: EagerStateMachine::new(tag_preview_handler),
            tag_preview_mode: true,
        }
    }

    #[inline]
    pub fn tokenize(&mut self, chunk: &Chunk) -> Result<usize, Error> {
        with_current_sm!(self, { sm.run_parsing_loop(chunk) })
    }

    #[cfg(feature = "testing_api")]
    pub fn tag_preview_mode(&mut self, is_enabled: bool) {
        self.tag_preview_mode = is_enabled;
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

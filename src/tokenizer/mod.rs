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

pub enum TagLexUnitResponse {
    SwitchToTagPreviewMode,
    None,
}

pub enum TagPreviewResponse {
    CaptureLexUnits,
    CaptureCurrentTagLexUnitOnly,
    None,
}

pub enum TokenizerOutputMode {
    LexUnits,
    TagPreviews,
    //NextLexUnitThenTagPreviews
}

declare_handler! {
    LexUnitHandler(&LexUnit)
}

// NOTE: we can switch between tokenizer modes on tags
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
    output_mode: TokenizerOutputMode,
}

// NOTE: dynamic dispatch can't be used for the StateMachine trait
// because it's not object-safe due to the usage of `Self` in function
// signatures, so we use this macro instead.
macro_rules! with_current_sm {
    ($self:tt, { sm.$fn:ident($($args:tt)*) }) => {
        match $self.output_mode {
            TokenizerOutputMode::TagPreviews => $self.eager_sm.$fn($($args)*),
            TokenizerOutputMode::LexUnits => $self.full_sm.$fn($($args)*),
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
        Tokenizer {
            full_sm: FullStateMachine::new(lex_unit_handler, tag_lex_unit_handler),
            eager_sm: EagerStateMachine::new(tag_preview_handler),
            output_mode: TokenizerOutputMode::TagPreviews,
        }
    }

    #[inline]
    pub fn tokenize(&mut self, chunk: &Chunk) -> Result<usize, Error> {
        with_current_sm!(self, { sm.run_parsing_loop(chunk) })
    }

    #[cfg(feature = "testing_api")]
    pub fn set_output_mode(&mut self, mode: TokenizerOutputMode) {
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

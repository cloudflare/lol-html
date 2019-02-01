/// Macro that implements accesors required by the StateMachine
/// trait and that are common for both implementations.
macro_rules! impl_common_sm_accessors {
    () => {
        #[inline]
        fn input_cursor(&mut self) -> &mut Cursor {
            &mut self.input_cursor
        }

        #[inline]
        fn set_input_cursor(&mut self, input_cursor: Cursor) {
            self.input_cursor = input_cursor;
        }

        #[inline]
        fn is_state_enter(&self) -> bool {
            self.is_state_enter
        }

        #[inline]
        fn set_is_state_enter(&mut self, val: bool) {
            self.is_state_enter = val;
        }

        #[inline]
        fn set_last_text_type(&mut self, text_type: TextType) {
            self.last_text_type = text_type;
        }

        #[inline]
        fn last_text_type(&self) -> TextType {
            self.last_text_type
        }

        #[inline]
        fn closing_quote(&self) -> u8 {
            self.closing_quote
        }

        #[inline]
        fn last_start_tag_name_hash(&self) -> Option<u64> {
            self.last_start_tag_name_hash
        }

        #[inline]
        fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
            self.last_start_tag_name_hash = name_hash;
        }

        #[inline]
        fn set_cdata_allowed(&mut self, cdata_allowed: bool) {
            self.cdata_allowed = cdata_allowed;
        }
    };
}

macro_rules! impl_common_sm_actions {
    () => {
        #[inline]
        fn set_closing_quote_to_double(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
            self.closing_quote = b'"';
        }

        #[inline]
        fn set_closing_quote_to_single(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
            self.closing_quote = b'\'';
        }

        #[inline]
        fn enter_cdata(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
            self.set_last_text_type(TextType::CDataSection);
        }

        #[inline]
        fn leave_cdata(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
            self.set_last_text_type(TextType::Data);
        }
    };
}

macro_rules! noop_action {
    ($($fn_name:ident),*) => {
        $(
            #[inline]
            fn $fn_name(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
                trace!(@noop);
            }
        )*
    };
}

pub mod eager;
pub mod full;

pub use self::eager::{EagerStateMachine, TagHintSink};
pub use self::full::{FullStateMachine, LexemeSink};

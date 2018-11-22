/// Macro that implements accesors required by the StateMachine
/// trait and that are common for both implementations.
macro_rules! impl_common_sm_accessors {
    () => {
        #[inline]
        fn get_input_cursor(&mut self) -> &mut Cursor {
            &mut self.input_cursor
        }

        #[inline]
        fn set_is_state_enter(&mut self, val: bool) {
            self.state_enter = val;
        }

        #[inline]
        fn is_state_enter(&self) -> bool {
            self.state_enter
        }

        #[inline]
        fn get_last_text_parsing_mode(&self) -> TextParsingMode {
            self.last_text_parsing_mode_change
        }

        #[inline]
        fn get_closing_quote(&self) -> u8 {
            self.closing_quote
        }

        #[inline]
        fn get_last_start_tag_name_hash(&self) -> Option<u64> {
            self.last_start_tag_name_hash
        }

        #[inline]
        fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
            self.last_start_tag_name_hash = name_hash;
        }

        #[inline]
        fn set_allow_cdata(&mut self, allow_cdata: bool) {
            self.allow_cdata = allow_cdata;
        }

        #[inline]
        fn set_input_cursor(&mut self, input_cursor: Cursor) {
            self.input_cursor = input_cursor;
        }
    };
}

macro_rules! noop_action {
    ($($fn_name:ident),*) => {
        $(
            #[inline]
            fn $fn_name(&mut self, _input: &Chunk, _ch: Option<u8>) { }
        )*
    };
}

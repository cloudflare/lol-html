use super::{LexemeSink, Lexer, TagTokenOutline};
use crate::parser::state_machine::StateMachineConditions;

impl<S: LexemeSink> StateMachineConditions for Lexer<S> {
    #[inline]
    fn is_appropriate_end_tag(&self) -> bool {
        match self.current_tag_token {
            Some(TagTokenOutline::EndTag { name_hash, .. }) => {
                self.last_start_tag_name_hash == name_hash
            }
            _ => {
                debug_assert!(false, "End tag should exist at this point");
                false
            }
        }
    }

    #[inline]
    fn cdata_allowed(&self) -> bool {
        self.cdata_allowed
    }
}

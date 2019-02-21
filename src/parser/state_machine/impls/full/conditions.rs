use super::*;
use crate::parser::state_machine::StateMachineConditions;

impl<S: LexemeSink> StateMachineConditions for FullStateMachine<S> {
    #[inline]
    fn is_appropriate_end_tag(&self, _ch: Option<u8>) -> bool {
        match self.current_tag_token {
            Some(TagTokenOutline::EndTag { name_hash, .. }) => {
                self.last_start_tag_name_hash == name_hash
            }
            _ => unreachable!("End tag should exist at this point"),
        }
    }

    #[inline]
    fn cdata_allowed(&self, _ch: Option<u8>) -> bool {
        self.cdata_allowed
    }
}

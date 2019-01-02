use super::*;
use crate::tokenizer::state_machine::StateMachineConditions;

impl<S: LexUnitSink> StateMachineConditions for FullStateMachine<S> {
    #[inline]
    fn is_appropriate_end_tag(&self, _ch: Option<u8>) -> bool {
        match self.current_token {
            Some(TokenView::EndTag { name_hash, .. }) => self.last_start_tag_name_hash == name_hash,
            _ => unreachable!("End tag should exist at this point"),
        }
    }

    #[inline]
    fn cdata_allowed(&self, _ch: Option<u8>) -> bool {
        self.cdata_allowed
    }
}

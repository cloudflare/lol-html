use super::*;
use crate::parser::state_machine::StateMachineConditions;

impl<S: TagHintSink> StateMachineConditions for EagerStateMachine<S> {
    #[inline]
    fn is_appropriate_end_tag(&self, _ch: Option<u8>) -> bool {
        self.tag_name_hash == self.last_start_tag_name_hash
    }

    #[inline]
    fn cdata_allowed(&self, _ch: Option<u8>) -> bool {
        self.cdata_allowed
    }
}

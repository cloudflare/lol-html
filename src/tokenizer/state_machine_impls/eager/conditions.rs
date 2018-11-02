use super::*;
use tokenizer::StateMachineConditions;

impl<H: TagPreviewHandler> StateMachineConditions for EagerStateMachine<H> {
    #[inline]
    fn is_appropriate_end_tag(&self, _ch: Option<u8>) -> bool {
        self.tag_name_hash == self.last_start_tag_name_hash
    }

    #[inline]
    fn cdata_allowed(&self, _ch: Option<u8>) -> bool {
        self.allow_cdata
    }
}

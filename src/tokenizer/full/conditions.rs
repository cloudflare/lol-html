use super::*;

impl<H: LexUnitHandler> Tokenizer<H> {
    #[inline]
    pub(super) fn is_appropriate_end_tag(&self) -> bool {
        match self.current_token {
            Some(TokenView::EndTag { name_hash, .. }) => self.last_start_tag_name_hash == name_hash,
            _ => unreachable!("End tag should exist at this point"),
        }
    }

    #[inline]
    pub(super) fn cdata_allowed(&self) -> bool {
        self.allow_cdata
    }
}

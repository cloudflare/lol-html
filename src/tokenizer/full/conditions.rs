use super::*;

impl<H: LexUnitHandler> Tokenizer<H> {
    #[inline]
    pub(super) fn is_appropriate_end_tag(&self, _ch: Option<u8>) -> bool {
        match self.current_token {
            Some(TokenView::EndTag { name_hash, .. }) => self.last_start_tag_name_hash == name_hash,
            _ => unreachable!("End tag should exist at this point"),
        }
    }

    #[inline]
    pub(super) fn cdata_allowed(&self, _ch: Option<u8>) -> bool {
        self.allow_cdata
    }

    #[inline]
    pub(super) fn is_closing_quote(&self, ch: Option<u8>) -> bool {
        match ch {
            Some(ch) if ch == self.closing_quote => true,
            _ => false,
        }
    }
}

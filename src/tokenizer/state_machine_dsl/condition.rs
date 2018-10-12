macro_rules! condition {
    (| $self:tt | > appropriate_end_tag) => {
        match $self.current_token {
            Some(TokenView::EndTag { name_hash, .. }) => {
                $self.last_start_tag_name_hash == name_hash
            }
            _ => unreachable!("End tag should exist at this point"),
        }
    };

    (| $self:tt | > cdata_allowed) => {
        $self.allow_cdata
    };
}

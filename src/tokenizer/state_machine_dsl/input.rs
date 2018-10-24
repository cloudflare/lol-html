macro_rules! input {
    ( @pos $self:tt ) => {
        $self.next_pos - 1
    };

    ( @consume_ch $self:tt, $input:ident ) => {{
        let ch = $input.get($self.next_pos);

        $self.next_pos += 1;

        trace!(@chars "consume", ch);

        ch
    }};

    ( @unconsume_ch $self:tt ) => {
        trace!(@chars "unconsume");

        $self.next_pos -= 1;
    };

    ( @consume_several $self:tt, $count:expr) => {
        trace!(@chars "consume several");

        $self.next_pos += $count;
    };

    ( @lookahead $self:tt, $input:ident, $offset:expr ) => {{
        let ch = $input.get($self.next_pos + $offset - 1);

        trace!(@chars "lookahead", ch);

        ch
    }};
}

define_state_group!(comment_states_group = {

    bogus_comment_state {
        b'>' => ( emit_comment; --> data_state )
        eof  => ( emit_comment; emit_eof; )
        _    => ()
    }

    comment_start_state <-- ( start_token_part; ) {
        // TODO
        eof  => ( emit_eof; )
        _    => ( emit_eof; )
    }

});

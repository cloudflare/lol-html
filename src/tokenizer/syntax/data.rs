define_state_group!(data_states_group = {

    data_state <-- ( mark_token_start; ) {
        b'<' => ( emit_char; mark_token_start; --> tag_open_state )
        eof  => ( emit_char; emit_eof; )
        _    => ()
    }

    tag_open_state {
        b'!'        => ( --> markup_declaration_open_state )
        b'/'        => ( --> end_tag_open_state )
        ascii-alpha => ()
        eof         => ( emit_char; emit_eof; )
        _           => ( emit_eof; )
    }

    markup_declaration_open_state {
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});

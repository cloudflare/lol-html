define_state_group!(data_states_group = {

    pub data_state <-- ( start_raw; ) {
        b'<' => ( emit_chars; start_raw; --> tag_open_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

    tag_open_state {
        b'!'  => ( --> markup_declaration_open_state )
        b'/'  => ( --> end_tag_open_state )
        alpha => ( create_start_tag; )
        eof   => ( emit_chars; emit_eof; )
        _     => ( emit_eof; )
    }

    markup_declaration_open_state {
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});

define_state_group!(data_states_group = {

    pub data_state <-- ( start_raw; ) {
        b'<' => ( emit_chars; start_raw; --> tag_open_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

    markup_declaration_open_state {
        // TODO
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});

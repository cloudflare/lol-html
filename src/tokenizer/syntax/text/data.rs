define_state_group!(data_states_group = {

    pub data_state <-- ( start_raw; notify_text_parsing_mode_change TextParsingMode::Data; ) {
        b'<' => ( emit_chars; start_raw; --> tag_open_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

});

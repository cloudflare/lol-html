define_state_group!(data_states_group = {

    data_state <-- ( notify_text_parsing_mode_change TextParsingMode::Data; ) {
        b'<' => ( emit_chars; --> tag_open_state )
        eoc  => ( emit_chars; )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

});

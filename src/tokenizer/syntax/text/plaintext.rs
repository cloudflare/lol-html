define_state_group!(plaintext_states_group = {

    pub plaintext_state <-- ( start_raw; notify_text_parsing_mode_change TextParsingMode::PlainText;) {
        eof => ( emit_chars; emit_eof; )
        _   => ()
    }

});

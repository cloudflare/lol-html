define_state_group!(plaintext_states_group = {

    plaintext_state <-- ( notify_text_parsing_mode_change TextParsingMode::PlainText;) {
        eoc => ( emit_chars; )
        eof => ( emit_chars; emit_eof; )
        _   => ()
    }

});

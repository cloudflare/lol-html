define_state_group!(cdata_section_states_group = {

    pub cdata_section_state <-- ( start_raw; notify_text_parsing_mode_change TextParsingMode::CDataSection; ) {
        b']' => ( emit_chars; start_raw; --> cdata_section_bracket_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

    cdata_section_bracket_state {
        [ "]>" ] => ( emit_raw_without_token; --> data_state )
        eof      => ( emit_chars; emit_eof; )
        _        => ( emit_chars; reconsume in cdata_section_state )
    }
});

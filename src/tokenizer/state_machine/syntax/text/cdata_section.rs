define_state_group!(cdata_section_states_group = {

    cdata_section_state {
        b']' => ( emit_chars; --> cdata_section_bracket_state )
        eoc  => ( emit_chars; )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

    cdata_section_bracket_state {
        [ "]>" ] => ( emit_raw_without_token; --> data_state )
        eof      => ( emit_chars; emit_eof; )
        _        => ( emit_chars; reconsume in cdata_section_state )
    }
});

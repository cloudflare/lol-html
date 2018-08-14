define_state_group!(cdata_section_states_group = {

    pub cdata_section_state <-- ( start_raw; ) {
        // TODO
        b']' => ( emit_chars; start_raw; --> cdata_section_bracket_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

    cdata_section_bracket_state {
        // TODO
        b']' => ( --> cdata_section_end_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ( emit_chars; reconsume in cdata_section_state )
    }

    cdata_section_end_state {
        // TODO
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});

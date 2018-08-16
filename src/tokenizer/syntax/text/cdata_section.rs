define_state_group!(cdata_section_states_group = {

    pub cdata_section_state <-- ( start_raw; ) {
        b']' => ( emit_chars; start_raw; mark_cdata_end; --> cdata_section_bracket_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

    cdata_section_bracket_state {
        b']' => ( --> cdata_section_end_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ( emit_chars; reconsume in cdata_section_state )
    }

    cdata_section_end_state {
        b']' => ( shift_cdata_end; )
        b'>' => ( emit_chars_up_to_cdata_end; --> data_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ( emit_chars; reconsume in cdata_section_state )
    }

});

define_state_group!(cdata_section_states_group = {

    pub cdata_section_state {
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});

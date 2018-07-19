define_state_group!(cdata_section_states_group = {

    cdata_section_state {
        >ch ( emit_eof; )
        >eof ( emit_eof; )
    }

});

define_state_group!(script_data_states_group = {

    script_data_state {
        >ch ( emit_eof; )
        >eof ( emit_eof; )
    }

});

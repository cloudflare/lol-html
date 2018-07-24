define_state_group!(script_data_states_group = {

    script_data_state {
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});

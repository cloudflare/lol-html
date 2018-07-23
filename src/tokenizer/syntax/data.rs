define_state_group!(data_states_group = {

    data_state {
        >eof ( emit_eof; )
        >ch ( emit_eof; )
    }

});

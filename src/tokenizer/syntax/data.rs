define_state_group!(data_states_group = {

    data_state {
        >ch ( emit_eof; )
        >eof ( emit_eof; )
    }

});

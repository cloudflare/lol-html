define_state_group!(rcdata_states_group = {

    rcdata_state {
        >ch ( emit_eof; )
        >eof ( emit_eof; )
    }

});

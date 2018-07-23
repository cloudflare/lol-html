define_state_group!(rawtext_states_group = {

    rawtext_state {
        >eof ( emit_eof; )
        >ch ( emit_eof; )
    }

});

define_state_group!(data_states_group = {

    data_state {
        b'<' => ( emit_chars; mark_tag_start; --> tag_open_state )
        eoc  => ( emit_chars; )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

});

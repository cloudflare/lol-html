define_state_group!(data_states_group = {

    data_state {
        b'<' => ( emit_char; --> tag_open_state )
        eof => ( emit_char; emit_eof; )
        _ => ()
    }

    tag_open_state {
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});
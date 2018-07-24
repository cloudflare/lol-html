define_state_group!(data_states_group = {

    data_state {
        --> ( create_char; )

        >eof ( emit_char; emit_eof; )

        >ch {
            b'<'    => (emit_char; --> tag_open_state)
            _       => ()
        }
    }

    tag_open_state {
        >eof( emit_eof; )
        >ch (emit_eof;)
    }

});


/*
data_state <-- ( create_char; ) {
        b'<' => (emit_char; --> tag_open_state)
        eof  => ( emit_char; emit_eof; )
        _    => ()
    }
*/
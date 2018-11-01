define_state_group!(rcdata_states_group = {

    rcdata_state <-- ( notify_text_parsing_mode_change TextParsingMode::RCData; ) {
        b'<' => ( emit_chars; --> rcdata_less_than_sign_state )
        eoc  => ( emit_chars; )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

    rcdata_less_than_sign_state {
        b'/' => ( --> rcdata_end_tag_open_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ( emit_chars; reconsume in rcdata_state )
    }

    rcdata_end_tag_open_state {
        alpha => ( create_end_tag; start_token_part; update_tag_name_hash; --> rcdata_end_tag_name_state )
        eof   => ( emit_chars; emit_eof; )
        _     => ( emit_chars; reconsume in rcdata_state )
    }

    rcdata_end_tag_name_state {
        whitespace => (
            if is_appropriate_end_tag
                ( finish_tag_name; --> before_attribute_name_state )
            else
                ( emit_chars; reconsume in rcdata_state )
        )

        b'/' => (
            if is_appropriate_end_tag
                ( finish_tag_name; --> self_closing_start_tag_state )
            else
                ( emit_chars; reconsume in rcdata_state )
        )

        b'>' => (
            if is_appropriate_end_tag
                ( finish_tag_name; emit_current_token; --> data_state )
            else
                ( emit_chars; reconsume in rcdata_state )
        )

        alpha => ( update_tag_name_hash; )
        eof   => ( emit_chars; emit_eof; )
        _     => ( emit_chars; reconsume in rcdata_state )
    }

});

#[macro_use]
mod escaped;

#[macro_use]
mod double_escaped;

define_state_group!(script_data_states_group = {

    pub script_data_state <-- ( start_raw; notify_text_parsing_mode_change TextParsingMode::ScriptData; ) {
        b'<' => ( emit_chars; start_raw; --> script_data_less_than_sign_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

    script_data_less_than_sign_state {
        b'/' => ( --> script_data_end_tag_open_state )
        b'!' => ( --> script_data_escape_start_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ( reconsume in script_data_state )
    }

    script_data_end_tag_open_state {
        alpha => ( create_end_tag; start_token_part; update_tag_name_hash; --> script_data_end_tag_name_state )
        eof   => ( emit_chars; emit_eof; )
        _     => ( emit_chars; reconsume in script_data_state )
    }

    script_data_end_tag_name_state {
        whitespace => (
            if appropriate_end_tag ( finish_tag_name; --> before_attribute_name_state )
            else ( emit_chars; reconsume in script_data_state )
        )

        b'/' => (
            if appropriate_end_tag ( finish_tag_name; --> self_closing_start_tag_state )
            else ( emit_chars; reconsume in script_data_state )
        )

        b'>' => (
            if appropriate_end_tag ( finish_tag_name; emit_current_token; --> data_state )
            else ( emit_chars; reconsume in script_data_state )
        )

        alpha => ( update_tag_name_hash; )
        eof   => ( emit_chars; emit_eof; )
        _     => ( emit_chars; reconsume in script_data_state )
    }

});

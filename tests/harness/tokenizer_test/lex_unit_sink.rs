use crate::harness::tokenizer_test::decoder::Decoder;
use crate::harness::tokenizer_test::test_outputs::TestToken;
use cool_thing::tokenizer::{LexUnit, TextParsingMode, TextParsingModeSnapshot, TokenView};

fn decode_text(text: &mut str, text_parsing_mode: TextParsingMode) -> String {
    let mut decoder = Decoder::new(text);

    if text_parsing_mode.should_replace_unsafe_null_in_text() {
        decoder = decoder.unsafe_null();
    }

    if text_parsing_mode.allows_text_entitites() {
        decoder = decoder.text_entities();
    }

    decoder.run()
}

#[derive(Default)]
pub struct LexUnitSink {
    pub tokens: Vec<TestToken>,
    pub raw_slices: Vec<Vec<u8>>,
    pub text_parsing_mode_snapshots: Vec<TextParsingModeSnapshot>,
    buffered_chars: Option<Vec<u8>>,
}

impl LexUnitSink {
    pub fn add_lex_unit(&mut self, lex_unit: &LexUnit<'_>, mode_snapshot: TextParsingModeSnapshot) {
        if let (Some(TokenView::Character), Some(raw)) = (lex_unit.token_view(), lex_unit.raw()) {
            self.buffer_chars(&raw, mode_snapshot);
        } else {
            if let Some(token) = lex_unit.as_token() {
                self.flush();
                self.tokens.push(TestToken::new(token, lex_unit));
                self.text_parsing_mode_snapshots.push(mode_snapshot);
            }

            if let Some(raw) = lex_unit.raw() {
                self.raw_slices.push(raw.to_vec());
            }
        }
    }

    pub fn flush(&mut self) {
        if let Some(buffered_chars) = self.buffered_chars.take() {
            let mut text = String::from_utf8(buffered_chars).unwrap();

            let mode_snapshot = self
                .text_parsing_mode_snapshots
                .last()
                .expect("Buffered chars should have associated mode snapshot");

            text = decode_text(&mut text, mode_snapshot.mode);
            self.tokens.push(TestToken::Character(text));
        }
    }

    fn buffer_chars(&mut self, chars: &[u8], mode_snapshot: TextParsingModeSnapshot) {
        if let Some(ref mut buffered_chars) = self.buffered_chars {
            buffered_chars.extend_from_slice(chars);

            if let Some(last_raw) = self.raw_slices.last_mut() {
                last_raw.extend_from_slice(chars);
            }
        } else {
            self.buffered_chars = Some(chars.to_vec());
            self.raw_slices.push(chars.to_vec());
            self.text_parsing_mode_snapshots.push(mode_snapshot);
        }
    }
}

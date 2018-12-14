use crate::harness::tokenizer_test::decoder::Decoder;
use crate::harness::tokenizer_test::test_outputs::TestToken;
use cool_thing::token::Token;
use cool_thing::tokenizer::{LexUnit, TextParsingMode, TextParsingModeSnapshot};
use encoding_rs::UTF_8;

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
    pub cummulative_raw: Vec<u8>,
    pub text_parsing_mode_snapshots: Vec<TextParsingModeSnapshot>,
    buffered_text: Option<Vec<u8>>,
}

impl LexUnitSink {
    pub fn add_lex_unit(&mut self, lex_unit: &LexUnit<'_>, mode_snapshot: TextParsingModeSnapshot) {
        if let Some(token) = Token::try_from(lex_unit, UTF_8) {
            if let Token::Text(t) = token {
                self.buffer_text(t.text(), mode_snapshot);
                return;
            } else {
                self.flush();

                if let Some(raw) = token.raw() {
                    self.raw_slices.push(raw.to_vec());
                }

                self.tokens.push(TestToken::new(token, lex_unit));
                self.text_parsing_mode_snapshots.push(mode_snapshot);
            }
        }

        let lex_unit_raw = lex_unit.input().slice(lex_unit.raw_range());

        self.cummulative_raw.extend_from_slice(&lex_unit_raw);
    }

    pub fn flush(&mut self) {
        if let Some(buffered_text) = self.buffered_text.take() {
            let mut text = String::from_utf8(buffered_text).unwrap();

            let mode_snapshot = self
                .text_parsing_mode_snapshots
                .last()
                .expect("Buffered text should have associated mode snapshot");

            text = decode_text(&mut text, mode_snapshot.mode);

            self.tokens.push(TestToken::Text(text));
        }
    }

    fn buffer_text(&mut self, text: &[u8], mode_snapshot: TextParsingModeSnapshot) {
        if let Some(ref mut buffered_text) = self.buffered_text {
            buffered_text.extend_from_slice(text);

            if let Some(last_raw) = self.raw_slices.last_mut() {
                last_raw.extend_from_slice(text);
            }
        } else {
            self.buffered_text = Some(text.to_vec());
            self.raw_slices.push(text.to_vec());
            self.text_parsing_mode_snapshots.push(mode_snapshot);
        }

        self.cummulative_raw.extend_from_slice(text);
    }
}

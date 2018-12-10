use crate::harness::tokenizer_test::decoder::Decoder;
use crate::harness::tokenizer_test::test_outputs::TestToken;
use cool_thing::rewriting::Token;
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
    buffered_text: Option<Vec<u8>>,
}

impl LexUnitSink {
    pub fn add_lex_unit(&mut self, lex_unit: &LexUnit<'_>, mode_snapshot: TextParsingModeSnapshot) {
        if let Some(TokenView::Text) = lex_unit.token_view() {
            self.buffer_text(lex_unit.raw(), mode_snapshot);
        } else {
            if let Some(token) = Token::try_from(lex_unit) {
                self.flush();
                self.tokens.push(TestToken::new(token, lex_unit));
                self.text_parsing_mode_snapshots.push(mode_snapshot);
            }

            self.raw_slices.push(lex_unit.raw().to_vec());
        }
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
    }
}

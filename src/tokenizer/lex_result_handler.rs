use super::{LexResult, Tokenizer};

pub type TokenizerStateAdjustment<H> = Option<fn(&mut Tokenizer<H>, Option<u8>)>;

pub trait LexResultHandler {
    fn handle<H: LexResultHandler>(&mut self, lex_res: LexResult) -> TokenizerStateAdjustment<H>;
}

#[cfg(feature = "testing_api")]
impl<F: FnMut(LexResult)> LexResultHandler for F {
    fn handle<H: LexResultHandler>(&mut self, lex_res: LexResult) -> TokenizerStateAdjustment<H> {
        self(lex_res);
        None
    }
}

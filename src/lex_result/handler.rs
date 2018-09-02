use super::LexResult;
use tokenizer::Tokenizer;

pub struct TokenizerAdjustment<H> {
    pub state: Option<fn(&mut Tokenizer<H>, Option<u8>)>,
    pub allow_cdata: bool,
}

pub trait LexResultHandlerWithFeedback {
    fn handle_and_provide_feedback<H: LexResultHandlerWithFeedback>(
        &mut self,
        lex_res: LexResult,
    ) -> Option<TokenizerAdjustment<H>>;
}

pub trait LexResultHandler {
    fn handle(&mut self, lex_res: LexResult);
}

impl<H: LexResultHandler> LexResultHandlerWithFeedback for H {
    fn handle_and_provide_feedback<F: LexResultHandlerWithFeedback>(
        &mut self,
        lex_res: LexResult,
    ) -> Option<TokenizerAdjustment<F>> {
        self.handle(lex_res);

        None
    }
}

#[cfg(feature = "testing_api")]
impl<F: FnMut(LexResult)> LexResultHandler for F {
    fn handle(&mut self, lex_res: LexResult) {
        self(lex_res);
    }
}

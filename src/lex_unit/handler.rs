use super::LexUnit;
use tokenizer::Tokenizer;

pub struct TokenizerAdjustment<H> {
    pub state: Option<fn(&mut Tokenizer<H>, Option<u8>)>,
    pub allow_cdata: bool,
}

pub trait LexUnitHandlerWithFeedback {
    fn handle_and_provide_feedback<H: LexUnitHandlerWithFeedback>(
        &mut self,
        lex_unit: LexUnit,
    ) -> Option<TokenizerAdjustment<H>>;
}

pub trait LexUnitHandler {
    fn handle(&mut self, lex_res: LexUnit);
}

impl<H: LexUnitHandler> LexUnitHandlerWithFeedback for H {
    fn handle_and_provide_feedback<F: LexUnitHandlerWithFeedback>(
        &mut self,
        lex_unit: LexUnit,
    ) -> Option<TokenizerAdjustment<F>> {
        self.handle(lex_unit);

        None
    }
}

#[cfg(feature = "testing_api")]
impl<F: FnMut(LexUnit)> LexUnitHandler for F {
    fn handle(&mut self, lex_unit: LexUnit) {
        self(lex_unit);
    }
}

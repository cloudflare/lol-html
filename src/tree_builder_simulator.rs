use tokenizer::{LexResult, LexResultHandler, LexResultHandlerWithFeedback, TokenizerAdjustment};

pub struct TreeBuilderSimulator<H> {
    lex_res_handler: H,
}

impl<H: LexResultHandler> TreeBuilderSimulator<H> {
    pub fn new(lex_res_handler: H) -> Self {
        TreeBuilderSimulator { lex_res_handler }
    }
}

impl<H: LexResultHandler> LexResultHandlerWithFeedback for TreeBuilderSimulator<H> {
    fn handle_and_provide_feedback<F: LexResultHandlerWithFeedback>(
        &mut self,
        lex_res: LexResult,
    ) -> Option<TokenizerAdjustment<F>> {
        self.lex_res_handler.handle(lex_res);

        None
    }
}

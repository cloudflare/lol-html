use lex_result::handler::*;
use lex_result::LexResult;

const DEFAULT_NS_STACK_CAPACITY: usize = 256;

enum Namespace {
    Html,
    Svg,
    MathML,
}

pub struct TreeBuilderSimulator<H> {
    lex_res_handler: H,
    ns_stack: Vec<Namespace>,
}

impl<H: LexResultHandler> TreeBuilderSimulator<H> {
    pub fn new(lex_res_handler: H) -> Self {
        TreeBuilderSimulator {
            lex_res_handler,
            ns_stack: Vec::with_capacity(DEFAULT_NS_STACK_CAPACITY),
        }
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

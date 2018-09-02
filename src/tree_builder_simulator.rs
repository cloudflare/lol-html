use lex_unit::handler::*;
use lex_unit::LexUnit;

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

impl<H: LexUnitHandler> TreeBuilderSimulator<H> {
    pub fn new(lex_res_handler: H) -> Self {
        TreeBuilderSimulator {
            lex_res_handler,
            ns_stack: Vec::with_capacity(DEFAULT_NS_STACK_CAPACITY),
        }
    }
}

impl<H: LexUnitHandler> LexUnitHandlerWithFeedback for TreeBuilderSimulator<H> {
    fn handle_and_provide_feedback<F: LexUnitHandlerWithFeedback>(
        &mut self,
        lex_res: LexUnit,
    ) -> Option<TokenizerAdjustment<F>> {
        self.lex_res_handler.handle(lex_res);

        None
    }
}

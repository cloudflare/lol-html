mod noop_tree_sink;
mod token_sink_proxy;

use self::noop_tree_sink::NoopTreeSink;
use self::token_sink_proxy::TokenSinkProxy;
use harness::tokenizer_test::TestToken;
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{BufferQueue, Tokenizer, TokenizerOpts, TokenizerResult};
use html5ever::tree_builder::{TreeBuilder, TreeBuilderOpts};

pub fn get_expected_tokens_with_feedback(input: &str) -> Vec<TestToken> {
    let mut tokens = Vec::new();
    let mut b = BufferQueue::new();

    b.push_back(StrTendril::from(input));

    {
        let mut t = Tokenizer::new(
            TokenSinkProxy {
                inner: TreeBuilder::new(NoopTreeSink::default(), TreeBuilderOpts::default()),
                tokens: &mut tokens,
            },
            TokenizerOpts::default(),
        );

        while let TokenizerResult::Script(_) = t.feed(&mut b) {
            // ignore script markers
        }

        t.end();
    }

    tokens
}

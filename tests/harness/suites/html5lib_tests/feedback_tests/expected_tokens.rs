use super::super::TestToken;
use hashbrown::HashMap;
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{
    BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
};
use html5ever::tree_builder::{TreeBuilder, TreeBuilderOpts};
use html5ever::TokenizerResult;
use markup5ever_rcdom::RcDom;
use std::cell::RefCell;
use std::iter::FromIterator;
use std::string::ToString;

// sends tokens to a given sink, while at the same time converting and
// recording them into the provided array
pub struct TokenSinkProxy<'a, Sink> {
    pub inner: Sink,
    pub tokens: RefCell<&'a mut Vec<TestToken>>,
}

impl<Sink> TokenSinkProxy<'_, Sink> {
    fn push_text_token(&self, s: &str) {
        let tokens = &mut **self.tokens.borrow_mut();
        if let Some(&mut TestToken::Text(ref mut last)) = tokens.last_mut() {
            *last += s;
        } else {
            tokens.push(TestToken::Text(s.to_string()));
        }
    }
}

impl<Sink: TokenSink> TokenSink for TokenSinkProxy<'_, Sink> {
    type Handle = Sink::Handle;

    fn process_token(&self, token: Token, line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            Token::DoctypeToken(ref doctype) => {
                self.tokens.borrow_mut().push(TestToken::Doctype {
                    name: doctype.name.as_ref().map(ToString::to_string),
                    public_id: doctype.public_id.as_ref().map(ToString::to_string),
                    system_id: doctype.system_id.as_ref().map(ToString::to_string),
                    force_quirks: doctype.force_quirks,
                });
            }
            Token::TagToken(ref tag) => {
                let name = tag.name.to_string();

                self.tokens.borrow_mut().push(match tag.kind {
                    TagKind::StartTag => TestToken::StartTag {
                        name,
                        attributes: HashMap::from_iter(
                            tag.attrs
                                .iter()
                                .rev()
                                .map(|attr| (attr.name.local.to_string(), attr.value.to_string())),
                        ),
                        self_closing: tag.self_closing,
                    },
                    TagKind::EndTag => TestToken::EndTag { name },
                });
            }
            Token::CommentToken(ref s) => {
                self.tokens
                    .borrow_mut()
                    .push(TestToken::Comment(s.to_string()));
            }
            Token::CharacterTokens(ref s) => {
                if !s.is_empty() {
                    self.push_text_token(s);
                }
            }
            Token::NullCharacterToken => {
                self.push_text_token("\0");
            }
            _ => {}
        }
        self.inner.process_token(token, line_number)
    }

    fn end(&self) {
        self.inner.end();
    }

    fn adjusted_current_node_present_but_not_in_html_namespace(&self) -> bool {
        self.inner
            .adjusted_current_node_present_but_not_in_html_namespace()
    }
}

pub fn get(input: &str) -> Vec<TestToken> {
    let mut tokens = Vec::default();
    let b = BufferQueue::default();

    b.push_back(StrTendril::from(input));

    {
        let t = Tokenizer::new(
            TokenSinkProxy {
                inner: TreeBuilder::new(RcDom::default(), TreeBuilderOpts::default()),
                tokens: RefCell::new(&mut tokens),
            },
            TokenizerOpts::default(),
        );

        while let TokenizerResult::Script(_) = t.feed(&b) {
            // ignore script markers
        }

        t.end();
    }

    tokens
}

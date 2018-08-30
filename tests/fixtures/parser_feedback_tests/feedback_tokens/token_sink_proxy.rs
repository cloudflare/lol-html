use cool_thing::get_tag_name_hash;
use harness::token::TestToken;
use html5ever::tokenizer::{TagKind, Token, TokenSink, TokenSinkResult};
use std::collections::HashMap;
use std::iter::FromIterator;

// sends tokens to a given sink, while at the same time converting and
// recording them into the provided array
pub struct TokenSinkProxy<'a, Sink> {
    pub inner: Sink,
    pub tokens: &'a mut Vec<TestToken>,
}

impl<'a, Sink> TokenSinkProxy<'a, Sink> {
    fn push_character_token(&mut self, s: &str) {
        if let Some(&mut TestToken::Character(ref mut last)) = self.tokens.last_mut() {
            *last += s;

            return;
        }
        self.tokens.push(TestToken::Character(s.to_string()));
    }
}

impl<'a, Sink> TokenSink for TokenSinkProxy<'a, Sink>
where
    Sink: TokenSink,
{
    type Handle = Sink::Handle;

    fn process_token(&mut self, token: Token, line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            Token::DoctypeToken(ref doctype) => {
                self.tokens.push(TestToken::Doctype {
                    name: doctype.name.as_ref().map(|s| s.to_string()),
                    public_id: doctype.public_id.as_ref().map(|s| s.to_string()),
                    system_id: doctype.system_id.as_ref().map(|s| s.to_string()),
                    force_quirks: doctype.force_quirks,
                });
            }
            Token::TagToken(ref tag) => {
                let name = tag.name.to_string();
                let name_hash = get_tag_name_hash(&name);

                self.tokens.push(match tag.kind {
                    TagKind::StartTag => TestToken::StartTag {
                        name,
                        name_hash,
                        attributes: HashMap::from_iter(
                            tag.attrs
                                .iter()
                                .rev()
                                .map(|attr| (attr.name.local.to_string(), attr.value.to_string())),
                        ),
                        self_closing: tag.self_closing,
                    },
                    TagKind::EndTag => TestToken::EndTag { name, name_hash },
                })
            }
            Token::CommentToken(ref s) => {
                self.tokens.push(TestToken::Comment(s.to_string()));
            }
            Token::CharacterTokens(ref s) => if !s.is_empty() {
                self.push_character_token(s);
            },
            Token::NullCharacterToken => {
                self.push_character_token("\0");
            }
            Token::EOFToken => {
                self.tokens.push(TestToken::Eof);
            }
            _ => {}
        }
        self.inner.process_token(token, line_number)
    }

    fn end(&mut self) {
        self.inner.end()
    }

    fn adjusted_current_node_present_but_not_in_html_namespace(&self) -> bool {
        self.inner
            .adjusted_current_node_present_but_not_in_html_namespace()
    }
}

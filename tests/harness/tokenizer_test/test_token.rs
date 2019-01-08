use super::decoder::{decode_attr_value, decode_text, to_null_decoded};
use super::unescape::Unescape;
use cool_thing::token::{Text, Token};
use cool_thing::tokenizer::TextParsingMode;
use cool_thing::transform_stream::TokenCaptureFlags;
use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde_json::error::Error;
use std::collections::HashMap;
use std::fmt::{self, Formatter};
use std::iter::FromIterator;

#[derive(Clone, Copy, Deserialize)]
enum TokenKind {
    Character,
    Comment,
    StartTag,
    EndTag,
    #[serde(rename = "DOCTYPE")]
    Doctype,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TestToken {
    Text(String),

    Comment(String),

    StartTag {
        name: String,
        attributes: HashMap<String, String>,
        self_closing: bool,
    },

    EndTag {
        name: String,
    },

    Doctype {
        name: Option<String>,
        public_id: Option<String>,
        system_id: Option<String>,
        force_quirks: bool,
    },

    Eof,
}

impl From<Token<'_>> for TestToken {
    fn from(token: Token<'_>) -> Self {
        match token {
            Token::Text(t) => match t {
                Text::Chunk(c) => TestToken::Text(c.text().into()),
                Text::End => TestToken::Text("".into()),
            },

            Token::Comment(t) => TestToken::Comment(to_null_decoded(&t.text())),

            Token::StartTag(t) => TestToken::StartTag {
                name: to_null_decoded(&t.name()),

                attributes: HashMap::from_iter(
                    t.attributes()
                        .iter()
                        .rev()
                        .map(|a| (to_null_decoded(&a.name()), decode_attr_value(&a.value()))),
                ),

                self_closing: t.self_closing(),
            },

            Token::EndTag(t) => TestToken::EndTag {
                name: to_null_decoded(&t.name()),
            },

            Token::Doctype(t) => TestToken::Doctype {
                name: t.name().map(|s| to_null_decoded(&s)),
                public_id: t.public_id().map(|s| to_null_decoded(&s)),
                system_id: t.system_id().map(|s| to_null_decoded(&s)),
                force_quirks: t.force_quirks(),
            },

            Token::Eof => TestToken::Eof,
        }
    }
}

impl<'de> Deserialize<'de> for TestToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> ::serde::de::Visitor<'de> for Visitor {
            type Value = TestToken;

            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("['TokenKind', ...]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: ::serde::de::SeqAccess<'de>,
            {
                let mut actual_length = 0;

                macro_rules! next {
                    ($error_msg:expr) => {
                        match seq.next_element()? {
                            Some(value) => {
                                #[allow(unused_assignments)]
                                #[allow(clippy::eval_order_dependence)]
                                {
                                    actual_length += 1;
                                }

                                value
                            }
                            None => return Err(DeError::invalid_length(actual_length, &$error_msg)),
                        }
                    };
                }

                let kind = next!("2 or more");

                Ok(match kind {
                    TokenKind::Character => TestToken::Text(next!("2")),
                    TokenKind::Comment => TestToken::Comment(next!("2")),
                    TokenKind::StartTag => TestToken::StartTag {
                        name: {
                            let mut value: String = next!("3 or 4");
                            value.make_ascii_lowercase();
                            value
                        },
                        attributes: {
                            let value: HashMap<String, String> = next!("3 or 4");
                            HashMap::from_iter(value.into_iter().map(|(mut k, v)| {
                                k.make_ascii_lowercase();
                                (k, v)
                            }))
                        },
                        self_closing: seq.next_element()?.unwrap_or(false),
                    },
                    TokenKind::EndTag => TestToken::EndTag {
                        name: {
                            let mut value: String = next!("2");
                            value.make_ascii_lowercase();
                            value
                        },
                    },
                    TokenKind::Doctype => TestToken::Doctype {
                        name: {
                            let mut value: Option<String> = next!("5");
                            if let Some(ref mut value) = value {
                                value.make_ascii_lowercase();
                            }
                            value
                        },
                        public_id: next!("5"),
                        system_id: next!("5"),
                        force_quirks: !next!("5"),
                    },
                })
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

impl Unescape for TestToken {
    fn unescape(&mut self) -> Result<(), Error> {
        match *self {
            TestToken::Text(ref mut s) | TestToken::Comment(ref mut s) => {
                s.unescape()?;
            }

            TestToken::EndTag { ref mut name, .. } => {
                name.unescape()?;
            }

            TestToken::StartTag {
                ref mut name,
                ref mut attributes,
                ..
            } => {
                name.unescape()?;

                for value in attributes.values_mut() {
                    value.unescape()?;
                }
            }

            TestToken::Doctype {
                ref mut name,
                ref mut public_id,
                ref mut system_id,
                ..
            } => {
                name.unescape()?;
                public_id.unescape()?;
                system_id.unescape()?;
            }
            TestToken::Eof => (),
        }
        Ok(())
    }
}

pub struct TestTokenList {
    tokens: Vec<TestToken>,
    text_parsing_mode: TextParsingMode,
}

impl Default for TestTokenList {
    fn default() -> Self {
        TestTokenList {
            tokens: Vec::new(),
            text_parsing_mode: TextParsingMode::Data,
        }
    }
}

impl TestTokenList {
    pub fn push(&mut self, token: Token<'_>, text_parsing_mode: TextParsingMode) {
        let token = TestToken::from(token);

        if let Some(TestToken::Text(last)) = self.tokens.last_mut() {
            if let TestToken::Text(ref curr) = token {
                *last += curr;
            } else {
                decode_text(last, self.text_parsing_mode);
                self.tokens.push(token);
            }
        } else {
            if let TestToken::Text(_) = token {
                self.text_parsing_mode = text_parsing_mode;
            }

            self.tokens.push(token);
        }
    }

    pub fn get_tokens(&self, capture_flags: TokenCaptureFlags) -> Vec<&TestToken> {
        self.tokens
            .iter()
            .filter(|t| match t {
                TestToken::Text(_) if capture_flags.contains(TokenCaptureFlags::TEXT) => true,
                TestToken::StartTag { .. }
                    if capture_flags.contains(TokenCaptureFlags::START_TAGS) =>
                {
                    true
                }
                TestToken::EndTag { .. } if capture_flags.contains(TokenCaptureFlags::END_TAGS) => {
                    true
                }
                TestToken::Doctype { .. }
                    if capture_flags.contains(TokenCaptureFlags::DOCTYPES) =>
                {
                    true
                }
                TestToken::Comment(_) if capture_flags.contains(TokenCaptureFlags::COMMENTS) => {
                    true
                }
                TestToken::Eof if capture_flags.contains(TokenCaptureFlags::EOF) => true,
                _ => false,
            })
            .collect()
    }
}

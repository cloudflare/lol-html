use super::decoder::{to_lower_null_decoded, to_null_decoded, Decoder};
use super::unescape::Unescape;
use cool_thing::tokenizer::{LexUnit, TagName, TagPreview, Token, TokenView};
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
    Character(String),

    Comment(String),

    StartTag {
        name: String,
        name_hash: Option<u64>,
        attributes: HashMap<String, String>,
        self_closing: bool,
    },

    EndTag {
        name: String,
        name_hash: Option<u64>,
    },

    Doctype {
        name: Option<String>,
        public_id: Option<String>,
        system_id: Option<String>,
        force_quirks: bool,
    },

    Eof,
}

impl TestToken {
    pub fn new(token: &Token<'_>, lex_unit: &LexUnit<'_>) -> Self {
        match token {
            Token::Character(t) => TestToken::Character(t.text().as_string()),
            Token::Comment(t) => TestToken::Comment(to_null_decoded(t.text())),

            Token::StartTag(t) => TestToken::StartTag {
                name: to_lower_null_decoded(t.name()),
                name_hash: match lex_unit.token_view() {
                    Some(&TokenView::StartTag { name_hash, .. }) => name_hash,
                    _ => None,
                },

                attributes: HashMap::from_iter(t.attributes().iter().rev().map(|a| {
                    (
                        to_lower_null_decoded(&a.name()),
                        Decoder::new(a.value().as_str())
                            .unsafe_null()
                            .attr_entities()
                            .run(),
                    )
                })),

                self_closing: t.self_closing(),
            },

            Token::EndTag(t) => TestToken::EndTag {
                name: to_lower_null_decoded(t.name()),
                name_hash: match lex_unit.token_view() {
                    Some(&TokenView::EndTag { name_hash, .. }) => name_hash,
                    _ => None,
                },
            },

            Token::Doctype(t) => TestToken::Doctype {
                name: t.name().map(to_lower_null_decoded),
                public_id: t.public_id().map(to_null_decoded),
                system_id: t.system_id().map(to_null_decoded),
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

                let mut token = match kind {
                    TokenKind::Character => TestToken::Character(next!("2")),
                    TokenKind::Comment => TestToken::Comment(next!("2")),
                    TokenKind::StartTag => TestToken::StartTag {
                        name: {
                            let mut value: String = next!("3 or 4");
                            value.make_ascii_lowercase();
                            value
                        },
                        name_hash: None,
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
                        name_hash: None,
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
                };

                match token {
                    TestToken::StartTag {
                        ref name,
                        ref mut name_hash,
                        ..
                    }
                    | TestToken::EndTag {
                        ref name,
                        ref mut name_hash,
                    } => {
                        *name_hash = TagName::get_hash(name);
                    }
                    _ => (),
                }

                Ok(token)
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

impl Unescape for TestToken {
    fn unescape(&mut self) -> Result<(), Error> {
        match *self {
            TestToken::Character(ref mut s) | TestToken::Comment(ref mut s) => {
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

#[derive(Debug, Copy, Clone)]
pub enum TagType {
    StartTag,
    EndTag,
}

#[derive(Debug)]
pub struct TestTagPreview {
    name: String,
    name_hash: Option<u64>,
    tag_type: TagType,
}

impl TestTagPreview {
    pub fn new(tag_preview: &TagPreview<'_>) -> Self {
        let mut tag_type = TagType::StartTag;

        let tag_name_info = match tag_preview {
            TagPreview::StartTag(name_info) => name_info,
            TagPreview::EndTag(name_info) => {
                tag_type = TagType::EndTag;
                name_info
            }
        };

        TestTagPreview {
            name: to_lower_null_decoded(tag_name_info.name()),
            name_hash: tag_name_info.name_hash,
            tag_type,
        }
    }
}

impl PartialEq<TestToken> for TestTagPreview {
    fn eq(&self, token: &TestToken) -> bool {
        match (self.tag_type, token) {
            (
                TagType::StartTag,
                TestToken::StartTag {
                    name, name_hash, ..
                },
            )
            | (
                TagType::EndTag,
                TestToken::EndTag {
                    name, name_hash, ..
                },
            ) => self.name == *name && self.name_hash == *name_hash,
            _ => false,
        }
    }
}

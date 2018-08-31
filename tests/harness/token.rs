use super::decoder::Decoder;
use super::unescape::Unescape;
use cool_thing::{get_tag_name_hash, LexResult, RawSubslice, ShallowToken, Token};
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

impl<'de> Deserialize<'de> for TestToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> ::serde::de::Visitor<'de> for Visitor {
            type Value = TestToken;

            fn expecting(&self, f: &mut Formatter) -> fmt::Result {
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
                                {
                                    actual_length += 1;
                                }

                                value
                            }
                            None => {
                                return Err(DeError::invalid_length(actual_length, &$error_msg))
                            }
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
                        *name_hash = get_tag_name_hash(name);
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

fn to_null_decoded(subslice: &RawSubslice) -> String {
    Decoder::new(subslice.as_str()).unsafe_null().run()
}

fn to_lower_null_decoded(subslice: &RawSubslice) -> String {
    let mut string = to_null_decoded(subslice);

    string.make_ascii_lowercase();

    string
}

impl<'r> From<(Token<'r>, &'r LexResult<'r>)> for TestToken {
    fn from((token, lex_res): (Token<'r>, &'r LexResult<'r>)) -> Self {
        match token {
            Token::Character(data) => TestToken::Character(data.as_string()),

            Token::Comment(ref data) => TestToken::Comment(to_null_decoded(data)),

            Token::StartTag {
                ref name,
                ref attributes,
                self_closing,
            } => TestToken::StartTag {
                name: to_lower_null_decoded(name),
                name_hash: match lex_res.shallow_token {
                    Some(ShallowToken::StartTag { name_hash, .. }) => name_hash,
                    _ => None,
                },

                attributes: HashMap::from_iter(attributes.iter().rev().map(|attr| {
                    (
                        to_lower_null_decoded(&attr.name),
                        Decoder::new(attr.value.as_str())
                            .unsafe_null()
                            .attr_entities()
                            .run(),
                    )
                })),

                self_closing,
            },

            Token::EndTag { ref name } => TestToken::EndTag {
                name: to_lower_null_decoded(name),
                name_hash: match lex_res.shallow_token {
                    Some(ShallowToken::EndTag { name_hash, .. }) => name_hash,
                    _ => None,
                },
            },

            Token::Doctype {
                name,
                public_id,
                system_id,
                force_quirks,
            } => TestToken::Doctype {
                name: name.as_ref().map(to_lower_null_decoded),
                public_id: public_id.as_ref().map(to_null_decoded),
                system_id: system_id.as_ref().map(to_null_decoded),
                force_quirks,
            },

            Token::Eof => TestToken::Eof,
        }
    }
}

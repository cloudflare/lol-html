use std::collections::HashMap;
use serde::de::{Deserialize, Deserializer, Error as DeError};
use std::fmt::{self, Formatter};
use std::iter::FromIterator;
use serde_json::error::Error;
use super::unescape::Unescape;
use cool_thing::Token;
use super::decoder::Decoder;

#[derive(Clone, Copy, Deserialize)]
enum TokenKind {
    Character,
    Comment,
    StartTag,
    EndTag,
    #[serde(rename = "DOCTYPE")] Doctype,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TestToken {
    Character(String),

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
                    ($error_msg: expr) => (match seq.next_element()? {
                        Some(value) => {
                            #[allow(unused_assignments)] {
                                actual_length += 1;
                            }

                            value
                        },
                        None => return Err(DeError::invalid_length(
                            actual_length,
                            &$error_msg
                        ))
                    })
                }

                let kind = next!("2 or more");

                Ok(match kind {
                    TokenKind::Character => TestToken::Character(next!("2")),
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
                        force_quirks: next!("5"),
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
            TestToken::Character(ref mut s) | TestToken::Comment(ref mut s) => {
                s.unescape()?;
            }

            TestToken::EndTag { ref mut name } => {
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

fn bytes_to_string(bytes: &[u8]) -> String {
    unsafe { String::from_utf8_unchecked(bytes.to_vec()) }
}

impl<'t> From<&'t Token<'t>> for TestToken {
    fn from(token: &Token<'t>) -> Self {
        match *token {
            Token::Character(data) => TestToken::Character(bytes_to_string(data)),

            Token::Comment(data) => TestToken::Comment(Decoder::new(data).unsafe_null().run()),

            Token::StartTag {
                name,
                attributes,
                self_closing,
            } => TestToken::StartTag {
                name: bytes_to_string(name),

                attributes: HashMap::from_iter(attributes.iter().rev().map(|attr| {
                    (
                        bytes_to_string(name),
                        Decoder::new(attr.value).unsafe_null().attr_entities().run(),
                    )
                })),

                self_closing,
            },

            Token::EndTag { name } => TestToken::EndTag {
                name: bytes_to_string(name),
            },

            Token::Doctype {
                name,
                public_id,
                system_id,
                force_quirks,
            } => TestToken::Doctype {
                name: name.map(bytes_to_string),
                public_id: public_id.map(bytes_to_string),
                system_id: system_id.map(bytes_to_string),
                force_quirks,
            },

            Token::Eof => TestToken::Eof,
        }
    }
}

use std::collections::HashMap;
use serde::de::{Deserialize, Deserializer, Error as DeError};
use std::fmt::{self, Formatter};
use std::iter::FromIterator;
use serde_json::error::Error;
use super::unescape::Unescape;
use cool_thing::{LexResult, TokenDescriptor, Token};
use super::decoder::Decoder;
use std::str;

#[derive(Clone, Copy, Deserialize)]
enum TokenKind {
    Character,
    Comment,
    StartTag,
    EndTag,
    #[serde(rename = "DOCTYPE")]
    Doctype,
}

#[derive(Deserialize)]
#[serde(remote = "Token")]
pub enum TokenDef {
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

impl<'de> Deserialize<'de> for TokenDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> ::serde::de::Visitor<'de> for Visitor {
            type Value = TokenDef;

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
                    TokenKind::Character => TokenDef::Character(next!("2")),
                    TokenKind::Comment => TokenDef::Comment(next!("2")),
                    TokenKind::StartTag => TokenDef::StartTag {
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
                    TokenKind::EndTag => TokenDef::EndTag {
                        name: {
                            let mut value: String = next!("2");
                            value.make_ascii_lowercase();
                            value
                        },
                    },
                    TokenKind::Doctype => TokenDef::Doctype {
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

impl Unescape for Token {
    fn unescape(&mut self) -> Result<(), Error> {
        match *self {
            Token::Character(ref mut s) | Token::Comment(ref mut s) => {
                s.unescape()?;
            }

            Token::EndTag { ref mut name } => {
                name.unescape()?;
            }

            Token::StartTag {
                ref mut name,
                ref mut attributes,
                ..
            } => {
                name.unescape()?;
                for value in attributes.values_mut() {
                    value.unescape()?;
                }
            }

            Token::Doctype {
                ref mut name,
                ref mut public_id,
                ref mut system_id,
                ..
            } => {
                name.unescape()?;
                public_id.unescape()?;
                system_id.unescape()?;
            }
            Token::Eof => (),
        }
        Ok(())
    }
}

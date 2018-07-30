mod token_descriptor;
mod token;

pub use self::token_descriptor::{AttributeDescriptor, TokenDescriptor};
pub use self::token::Token;
use std::collections::HashMap;
use std::iter::FromIterator;

fn bytes_to_string(bytes: &[u8]) -> String {
    unsafe { String::from_utf8_unchecked(bytes.to_vec()) }
}

pub struct LexResult<'r, 't: 'r> {
    pub token_descr: TokenDescriptor<'t>,
    pub raw: Option<&'r [u8]>,
}

impl<'r, 't> Into<Token> for LexResult<'r, 't> {
    fn into(self) -> Token {
        match (self.token_descr, self.raw) {
            (TokenDescriptor::Character, Some(raw)) => Token::Character(bytes_to_string(raw)),
            (TokenDescriptor::Comment, Some(raw)) => Token::Comment(bytes_to_string(raw)),

            (
                TokenDescriptor::StartTag {
                    name,
                    attributes,
                    self_closing,
                },
                Some(raw),
            ) => Token::StartTag {
                name: name.as_string(raw),

                attributes: HashMap::from_iter(
                    attributes
                        .iter()
                        .rev()
                        .map(|attr| (name.as_string(raw), attr.value.as_string(raw))),
                ),

                self_closing,
            },

            (TokenDescriptor::EndTag { name }, Some(raw)) => Token::EndTag {
                name: name.as_string(raw),
            },

            (
                TokenDescriptor::Doctype {
                    name,
                    public_id,
                    system_id,
                    force_quirks,
                },
                Some(raw),
            ) => Token::Doctype {
                name: name.as_ref().map(|s| s.as_string(raw)),
                public_id: public_id.as_ref().map(|s| s.as_string(raw)),
                system_id: system_id.as_ref().map(|s| s.as_string(raw)),
                force_quirks,
            },

            (TokenDescriptor::Eof, None) => Token::Eof,
            _ => unreachable!(),
        }
    }
}

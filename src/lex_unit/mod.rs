pub mod handler;
mod raw_subslice;
mod token;

pub use self::raw_subslice::RawSubslice;
pub use self::token::*;

#[inline]
fn as_opt_subslice(raw: &[u8], range: Option<SliceRange>) -> Option<RawSubslice> {
    range.map(|range| RawSubslice::from((raw, range)))
}

pub struct LexUnit<'r> {
    pub shallow_token: Option<ShallowToken>,
    pub raw: Option<&'r [u8]>,
}

impl<'r> LexUnit<'r> {
    pub fn as_token(&self) -> Option<Token<'r>> {
        self.shallow_token
            .as_ref()
            .map(|shallow_token| match (shallow_token, self.raw) {
                (ShallowToken::Character, Some(raw)) => Token::Character(RawSubslice::from(raw)),

                (&ShallowToken::Comment(text), Some(raw)) => {
                    Token::Comment(RawSubslice::from((raw, text)))
                }

                (
                    &ShallowToken::StartTag {
                        name,
                        ref attributes,
                        self_closing,
                        ..
                    },
                    Some(raw),
                ) => Token::StartTag {
                    name: RawSubslice::from((raw, name)),

                    attributes: attributes
                        .borrow()
                        .iter()
                        .map(|&ShallowAttribute { name, value }| Attribute {
                            name: RawSubslice::from((raw, name)),
                            value: RawSubslice::from((raw, value)),
                        })
                        .collect(),

                    self_closing,
                },

                (&ShallowToken::EndTag { name, .. }, Some(raw)) => Token::EndTag {
                    name: RawSubslice::from((raw, name)),
                },

                (
                    &ShallowToken::Doctype {
                        name,
                        public_id,
                        system_id,
                        force_quirks,
                    },
                    Some(raw),
                ) => Token::Doctype {
                    name: as_opt_subslice(raw, name),
                    public_id: as_opt_subslice(raw, public_id),
                    system_id: as_opt_subslice(raw, system_id),
                    force_quirks,
                },

                (ShallowToken::Eof, None) => Token::Eof,
                _ => unreachable!("Such a combination of raw value and token type shouldn't exist"),
            })
    }
}

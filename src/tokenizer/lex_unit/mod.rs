mod raw_subslice;
mod token;

pub use self::raw_subslice::RawSubslice;
pub use self::token::*;

#[inline]
fn as_opt_subslice(raw: &[u8], range: Option<SliceRange>) -> Option<RawSubslice> {
    range.map(|range| RawSubslice::from((raw, range)))
}

pub struct LexUnit<'r> {
    pub token_view: Option<TokenView>,
    pub raw: Option<&'r [u8]>,
}

impl<'r> LexUnit<'r> {
    pub fn as_token(&self) -> Option<Token<'r>> {
        self.token_view
            .as_ref()
            .map(|token_view| match (token_view, self.raw) {
                (TokenView::Character, Some(raw)) => Token::Character(RawSubslice::from(raw)),

                (&TokenView::Comment(text), Some(raw)) => {
                    Token::Comment(RawSubslice::from((raw, text)))
                }

                (
                    &TokenView::StartTag {
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
                        .map(|&AttributeView { name, value }| Attribute {
                            name: RawSubslice::from((raw, name)),
                            value: RawSubslice::from((raw, value)),
                        }).collect(),
                    self_closing,
                },

                (&TokenView::EndTag { name, .. }, Some(raw)) => Token::EndTag {
                    name: RawSubslice::from((raw, name)),
                },

                (
                    &TokenView::Doctype {
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

                (TokenView::Eof, None) => Token::Eof,
                _ => unreachable!("Such a combination of raw value and token type shouldn't exist"),
            })
    }
}

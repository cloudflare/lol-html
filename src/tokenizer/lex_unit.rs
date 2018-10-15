use base::{Bytes, Chunk, Range};
use lazycell::LazyCell;
pub use tokenizer::token::*;

pub struct LexUnit<'b> {
    input_chunk: &'b Chunk<'b>,
    raw_range: Option<Range>,
    token_view: Option<TokenView>,
    raw: LazyCell<Option<Bytes<'b>>>,
    token: LazyCell<Option<Token<'b>>>,
}

impl<'b> LexUnit<'b> {
    pub fn new(
        input_chunk: &'b Chunk<'b>,
        token_view: Option<TokenView>,
        raw_range: Option<Range>,
    ) -> Self {
        LexUnit {
            input_chunk,
            raw_range,
            token_view,
            raw: LazyCell::new(),
            token: LazyCell::new(),
        }
    }

    #[inline]
    fn get_opt_input_slice(&self, range: Option<Range>) -> Option<Bytes<'b>> {
        range.map(|range| self.input_chunk.slice(range))
    }

    pub fn get_raw(&self) -> Option<&Bytes<'b>> {
        self.raw
            .borrow_with(|| self.get_opt_input_slice(self.raw_range))
            .as_ref()
    }

    pub fn get_token_view(&self) -> Option<&TokenView> {
        self.token_view.as_ref()
    }

    pub fn get_token(&self) -> Option<&Token<'b>> {
        self.token
            .borrow_with(|| {
                self.token_view.as_ref().map(|token_view| match token_view {
                    TokenView::Character => Token::Character(
                        self.input_chunk.slice(
                            self.raw_range
                                .expect("Character token should always has raw representation"),
                        ),
                    ),
                    &TokenView::Comment(text) => Token::Comment(self.input_chunk.slice(text)),

                    &TokenView::StartTag {
                        name,
                        ref attributes,
                        self_closing,
                        ..
                    } => Token::StartTag(StartTagToken::new(
                        self.input_chunk,
                        self.input_chunk.slice(name),
                        attributes,
                        self_closing,
                    )),

                    &TokenView::EndTag { name, .. } => Token::EndTag {
                        name: self.input_chunk.slice(name),
                    },

                    &TokenView::Doctype {
                        name,
                        public_id,
                        system_id,
                        force_quirks,
                    } => Token::Doctype {
                        name: self.get_opt_input_slice(name),
                        public_id: self.get_opt_input_slice(public_id),
                        system_id: self.get_opt_input_slice(system_id),
                        force_quirks,
                    },

                    TokenView::Eof => Token::Eof,
                })
            }).as_ref()
    }
}

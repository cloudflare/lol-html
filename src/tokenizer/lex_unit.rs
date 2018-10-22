use base::{Bytes, IterableChunk, Range};
use lazycell::LazyCell;
pub use tokenizer::token::*;

pub struct LexUnit<'c> {
    input_chunk: &'c IterableChunk<'c>,
    raw_range: Option<Range>,
    token_view: Option<TokenView>,
    raw: LazyCell<Option<Bytes<'c>>>,
    token: LazyCell<Option<Token<'c>>>,
}

impl<'c> LexUnit<'c> {
    pub fn new(
        input_chunk: &'c IterableChunk<'c>,
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

    pub fn get_raw(&self) -> Option<&Bytes<'c>> {
        self.raw
            .borrow_with(|| self.input_chunk.opt_slice(self.raw_range))
            .as_ref()
    }

    pub fn get_token_view(&self) -> Option<&TokenView> {
        self.token_view.as_ref()
    }

    pub fn get_token(&self) -> Option<&Token<'c>> {
        self.token
            .borrow_with(|| {
                self.token_view.as_ref().map(|token_view| match token_view {
                    TokenView::Character => Token::Character(
                        self.input_chunk.slice(
                            self.raw_range
                                .expect("Character token should always have raw representation"),
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
                        name: self.input_chunk.opt_slice(name),
                        public_id: self.input_chunk.opt_slice(public_id),
                        system_id: self.input_chunk.opt_slice(system_id),
                        force_quirks,
                    },

                    TokenView::Eof => Token::Eof,
                })
            }).as_ref()
    }
}

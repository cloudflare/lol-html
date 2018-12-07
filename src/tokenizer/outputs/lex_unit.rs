use super::token::*;
use super::token_view::*;
use crate::base::{Bytes, Chunk, Range};
use lazycell::LazyCell;
use std::fmt::{self, Debug, Write};
use std::rc::Rc;

pub struct LexUnit<'c> {
    input: &'c Chunk<'c>,
    raw_range: Option<Range>,
    token_view: Option<TokenView>,
    raw: LazyCell<Option<Bytes<'c>>>,
    token: LazyCell<Option<Token<'c>>>,
}

impl<'c> LexUnit<'c> {
    pub fn new(
        input: &'c Chunk<'c>,
        token_view: Option<TokenView>,
        raw_range: Option<Range>,
    ) -> Self {
        LexUnit {
            input,
            raw_range,
            token_view,
            raw: LazyCell::new(),
            token: LazyCell::new(),
        }
    }

    pub fn get_raw(&self) -> Option<&Bytes<'c>> {
        self.raw
            .borrow_with(|| self.input.opt_slice(self.raw_range))
            .as_ref()
    }

    #[inline]
    pub fn get_token_view(&self) -> Option<&TokenView> {
        self.token_view.as_ref()
    }

    #[inline]
    pub fn get_raw_range(&self) -> Option<Range> {
        self.raw_range
    }

    pub fn get_token(&self) -> Option<&Token<'c>> {
        self.token
            .borrow_with(|| {
                self.token_view.as_ref().map(|token_view| match token_view {
                    TokenView::Character => Token::new_character(
                        self.input.slice(
                            self.raw_range
                                .expect("Character token should always have raw representation"),
                        ),
                    ),

                    &TokenView::Comment(text) => Token::new_comment(self.input.slice(text)),

                    &TokenView::StartTag {
                        name,
                        ref attributes,
                        self_closing,
                        ..
                    } => Token::new_start_tag(
                        self.input.slice(name),
                        AttributeList::new(self.input, Rc::clone(&attributes)),
                        self_closing,
                    ),

                    &TokenView::EndTag { name, .. } => Token::new_end_tag(self.input.slice(name)),

                    &TokenView::Doctype {
                        name,
                        public_id,
                        system_id,
                        force_quirks,
                    } => Token::new_doctype(
                        self.input.opt_slice(name),
                        self.input.opt_slice(public_id),
                        self.input.opt_slice(system_id),
                        force_quirks,
                    ),

                    TokenView::Eof => Token::Eof,
                })
            })
            .as_ref()
    }
}

impl Debug for LexUnit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("LexUnit");

        if let Some(raw_range) = self.raw_range {
            let mut pretty_raw = self.input.as_string();
            let mut start = String::new();
            let mut end = String::new();

            write!(start, "|{}|", raw_range.start)?;
            write!(end, "|{}|", raw_range.end)?;

            pretty_raw.insert_str(raw_range.end, &end);
            pretty_raw.insert_str(raw_range.start, &start);

            builder.field("raw", &format_args!("`{}`", &pretty_raw));
        }

        if let (Some(token_view), Some(token)) = (self.token_view.as_ref(), self.get_token()) {
            builder
                .field("token_view", token_view)
                .field("token", token);
        }

        builder.finish()
    }
}

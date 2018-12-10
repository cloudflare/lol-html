use super::token::*;
use super::token_view::*;
use crate::base::{Bytes, Chunk, Range};
use lazycell::LazyCell;
use std::fmt::{self, Debug, Write};
use std::rc::Rc;

pub struct LexUnit<'i> {
    input: &'i Chunk<'i>,
    raw_range: Range,
    token_view: Option<TokenView>,
    raw: LazyCell<Bytes<'i>>,
    token: LazyCell<Option<Token<'i>>>,
}

impl<'i> LexUnit<'i> {
    pub fn new(input: &'i Chunk<'i>, token_view: Option<TokenView>, raw_range: Range) -> Self {
        LexUnit {
            input,
            raw_range,
            token_view,
            raw: LazyCell::new(),
            token: LazyCell::new(),
        }
    }

    #[inline]
    pub fn raw(&self) -> &Bytes<'i> {
        self.raw.borrow_with(|| self.input.slice(self.raw_range))
    }

    #[inline]
    pub fn token_view(&self) -> Option<&TokenView> {
        self.token_view.as_ref()
    }

    #[inline]
    pub fn raw_range(&self) -> Range {
        self.raw_range
    }

    pub fn as_token(&self) -> Option<&Token<'i>> {
        self.token
            .borrow_with(|| {
                self.token_view.as_ref().map(|token_view| match token_view {
                    TokenView::Text => Token::new_text(self.input.slice(self.raw_range)),

                    &TokenView::Comment(text) => Token::new_comment(self.input.slice(text)),

                    &TokenView::StartTag {
                        name,
                        ref attributes,
                        self_closing,
                        ..
                    } => Token::new_start_tag(
                        self.input.slice(name),
                        ParsedAttributeList::new(self.input, Rc::clone(&attributes)),
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
        let mut pretty_raw = self.input.as_string();
        let mut start = String::new();
        let mut end = String::new();

        write!(start, "|{}|", self.raw_range.start)?;
        write!(end, "|{}|", self.raw_range.end)?;

        pretty_raw.insert_str(self.raw_range.end, &end);
        pretty_raw.insert_str(self.raw_range.start, &start);

        builder.field("raw", &format_args!("`{}`", &pretty_raw));

        if let (Some(token_view), Some(token)) = (self.token_view.as_ref(), self.as_token()) {
            builder
                .field("token_view", token_view)
                .field("token", token);
        }

        builder.finish()
    }
}

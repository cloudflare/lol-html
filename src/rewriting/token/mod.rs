mod comment;
mod doctype;
mod end_tag;
mod start_tag;
mod text;

use self::start_tag::ParsedAttributeList;
use crate::tokenizer::{LexUnit, TokenView};
use std::rc::Rc;

pub use self::comment::Comment;
pub use self::doctype::Doctype;
pub use self::end_tag::EndTag;
pub use self::start_tag::{Attribute, Attributes, StartTag};
pub use self::text::Text;

#[derive(Debug)]
pub enum Token<'i> {
    Text(Text<'i>),
    Comment(Comment<'i>),
    StartTag(StartTag<'i>),
    EndTag(EndTag<'i>),
    Doctype(Doctype<'i>),
    Eof,
}

impl<'l, 'i: 'l> Token<'i> {
    pub fn try_from(lex_unit: &'l LexUnit<'i>) -> Option<Token<'l>> {
        let input = lex_unit.input();

        lex_unit.token_view().map(|token_view| match token_view {
            TokenView::Text => Token::Text(Text::new_parsed(input.slice(lex_unit.raw_range()))),

            &TokenView::Comment(text) => Token::Comment(Comment::new_parsed(input.slice(text))),

            &TokenView::StartTag {
                name,
                ref attributes,
                self_closing,
                ..
            } => Token::StartTag(StartTag::new_parsed(
                input.slice(name),
                ParsedAttributeList::new(input, Rc::clone(&attributes)),
                self_closing,
            )),

            &TokenView::EndTag { name, .. } => Token::EndTag(EndTag::new_parsed(input.slice(name))),

            &TokenView::Doctype {
                name,
                public_id,
                system_id,
                force_quirks,
            } => Token::Doctype(Doctype::new_parsed(
                input.opt_slice(name),
                input.opt_slice(public_id),
                input.opt_slice(system_id),
                force_quirks,
            )),

            TokenView::Eof => Token::Eof,
        })
    }
}

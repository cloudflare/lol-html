use super::token_view::*;
use crate::base::{Bytes, Chunk, Range};
use std::fmt::{self, Debug, Write};

pub struct LexUnit<'i> {
    input: &'i Chunk<'i>,
    raw_range: Range,
    token_view: Option<TokenView>,
}

impl<'i> LexUnit<'i> {
    pub fn new(input: &'i Chunk<'i>, token_view: Option<TokenView>, raw_range: Range) -> Self {
        LexUnit {
            input,
            raw_range,
            token_view,
        }
    }

    #[inline]
    pub fn input(&self) -> &Chunk<'i> {
        self.input
    }

    #[inline]
    pub fn token_view(&self) -> Option<&TokenView> {
        self.token_view.as_ref()
    }

    #[inline]
    pub fn raw_range(&self) -> Range {
        self.raw_range
    }

    #[inline]
    pub fn part(&self, range: Range) -> Bytes<'_> {
        self.input.slice(range)
    }

    #[inline]
    pub fn opt_part(&self, range: Option<Range>) -> Option<Bytes<'_>> {
        self.input.opt_slice(range)
    }

    #[inline]
    pub fn raw(&self) -> Bytes<'_> {
        self.input.slice(self.raw_range())
    }
}

impl<'i> Debug for LexUnit<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("LexUnit");
        let mut pretty_raw = self.input.as_debug_string();
        let mut start = String::new();
        let mut end = String::new();

        write!(start, "|{}|", self.raw_range.start)?;
        write!(end, "|{}|", self.raw_range.end)?;

        pretty_raw.insert_str(self.raw_range.end, &end);
        pretty_raw.insert_str(self.raw_range.start, &start);

        builder.field("raw", &format_args!("`{}`", &pretty_raw));

        if let Some(token_view) = self.token_view() {
            builder.field("token_view", token_view);
        }

        builder.finish()
    }
}

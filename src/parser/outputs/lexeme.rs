use super::token_outline::*;
use crate::base::{Bytes, Chunk, Range};
use std::fmt::{self, Debug, Write};

pub struct Lexeme<'i> {
    input: &'i Chunk<'i>,
    raw_range: Range,
    token_outline: Option<TokenOutline>,
}

impl<'i> Lexeme<'i> {
    pub fn new(
        input: &'i Chunk<'i>,
        token_outline: Option<TokenOutline>,
        raw_range: Range,
    ) -> Self {
        Lexeme {
            input,
            raw_range,
            token_outline,
        }
    }

    #[inline]
    pub fn input(&self) -> &Chunk<'i> {
        self.input
    }

    #[inline]
    pub fn token_outline(&self) -> Option<&TokenOutline> {
        self.token_outline.as_ref()
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

impl Debug for Lexeme<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("Lexeme");
        let mut pretty_raw = self.input.as_debug_string();
        let mut start = String::new();
        let mut end = String::new();

        write!(start, "|{}|", self.raw_range.start)?;
        write!(end, "|{}|", self.raw_range.end)?;

        pretty_raw.insert_str(self.raw_range.end, &end);
        pretty_raw.insert_str(self.raw_range.start, &start);

        builder.field("raw", &format_args!("`{}`", &pretty_raw));

        if let Some(token_outline) = self.token_outline() {
            builder.field("token_outline", token_outline);
        }

        builder.finish()
    }
}

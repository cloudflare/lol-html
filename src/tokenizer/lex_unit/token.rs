use super::raw_subslice::RawSubslice;
use std::cell::RefCell;
use std::rc::Rc;

// NOTE: std::ops::Range implements iterator and, thus, doesn't implement Copy.
// See: https://github.com/rust-lang/rust/pull/27186
#[derive(Clone, Copy, Default, Debug)]
pub struct SliceRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Default)]
pub struct AttributeView {
    pub name: SliceRange,
    pub value: SliceRange,
}

#[cfg_attr(feature = "testing_api", derive(Debug))]
pub struct Attribute<'r> {
    pub name: RawSubslice<'r>,
    pub value: RawSubslice<'r>,
}

#[derive(Debug)]
pub enum TokenView {
    Character,

    Comment(SliceRange),

    StartTag {
        name: SliceRange,
        name_hash: Option<u64>,
        attributes: Rc<RefCell<Vec<AttributeView>>>,
        self_closing: bool,
    },

    EndTag {
        name: SliceRange,
        name_hash: Option<u64>,
    },

    Doctype {
        name: Option<SliceRange>,
        public_id: Option<SliceRange>,
        system_id: Option<SliceRange>,
        force_quirks: bool,
    },

    Eof,
}

#[cfg_attr(feature = "testing_api", derive(Debug))]
pub enum Token<'r> {
    Character(RawSubslice<'r>),

    Comment(RawSubslice<'r>),

    StartTag {
        name: RawSubslice<'r>,
        attributes: Vec<Attribute<'r>>,
        self_closing: bool,
    },

    EndTag {
        name: RawSubslice<'r>,
    },

    Doctype {
        name: Option<RawSubslice<'r>>,
        public_id: Option<RawSubslice<'r>>,
        system_id: Option<RawSubslice<'r>>,
        force_quirks: bool,
    },

    Eof,
}

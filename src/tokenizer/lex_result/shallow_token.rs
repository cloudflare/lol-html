use std::cell::RefCell;
use std::rc::Rc;

// NOTE: std::ops::Range implements iterator and, thus, doesn't implement Copy.
// See: https://github.com/rust-lang/rust/pull/27186
#[derive(Clone, Copy, Default, Debug)]
pub struct SliceRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub struct ShallowAttribute {
    pub name: SliceRange,
    pub value: SliceRange,
}

#[derive(Debug)]
pub enum ShallowToken {
    Character,

    Comment,

    StartTag {
        name: SliceRange,
        attributes: Rc<RefCell<Vec<ShallowAttribute>>>,
        self_closing: bool,
    },

    EndTag {
        name: SliceRange,
    },

    Doctype {
        name: Option<SliceRange>,
        public_id: Option<SliceRange>,
        system_id: Option<SliceRange>,
        force_quirks: bool,
    },

    Eof,
}

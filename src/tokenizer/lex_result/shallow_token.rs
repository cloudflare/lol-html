// NOTE: std::ops::Range implements iterator and, thus, doesn't implement Copy.
// See: https://github.com/rust-lang/rust/pull/27186
#[derive(Clone, Copy)]
pub struct SliceRange {
    pub start: usize,
    pub end: usize,
}

pub struct ShallowAttribute {
    pub name: SliceRange,
    pub value: SliceRange,
}

pub enum ShallowToken<'t> {
    Character,

    Comment,

    StartTag {
        name: SliceRange,
        attributes: &'t [ShallowAttribute],
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

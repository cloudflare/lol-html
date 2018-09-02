use super::raw_subslice::RawSubslice;

#[cfg_attr(feature = "testing_api", derive(Debug))]
pub struct Attribute<'r> {
    pub name: RawSubslice<'r>,
    pub value: RawSubslice<'r>,
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

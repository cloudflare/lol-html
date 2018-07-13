#[derive(Debug)]
pub struct Attribute<'t> {
    pub name: &'t [u8],
    pub value: &'t [u8],
}

#[derive(Debug)]
pub enum Token<'t> {
    Character(&'t [u8]),

    Comment(&'t [u8]),

    StartTag {
        name: &'t [u8],
        attributes: &'t [Attribute<'t>],
        self_closing: bool,
    },

    EndTag {
        name: &'t [u8],
    },

    Doctype {
        name: Option<&'t [u8]>,
        public_id: Option<&'t [u8]>,
        system_id: Option<&'t [u8]>,
        force_quirks: bool,
    },

    Eof,
}

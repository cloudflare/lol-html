use super::buffer_slice::BufferSlice;

#[derive(Debug)]
pub struct Attribute<'t> {
    pub name: BufferSlice<'t>,
    pub value: BufferSlice<'t>,
}

#[derive(Debug)]
pub enum Token<'t> {
    Character(BufferSlice<'t>),

    Comment(BufferSlice<'t>),

    StartTag {
        name: BufferSlice<'t>,
        attributes: &'t [Attribute<'t>],
        self_closing: bool,
    },

    EndTag {
        name: BufferSlice<'t>,
    },

    Doctype {
        name: Option<BufferSlice<'t>>,
        public_id: Option<BufferSlice<'t>>,
        system_id: Option<BufferSlice<'t>>,
        force_quirks: bool,
    },

    Eof,
}

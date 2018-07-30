use std::str;
use super::bytes_to_string;

pub struct RawSubslice {
    start: usize,
    end: usize,
}

impl RawSubslice {
    pub fn as_bytes<'r>(&self, raw: &'r [u8]) -> &'r [u8] {
        &raw[self.start..self.end]
    }

    pub fn as_str<'t>(&self, raw: &'t [u8]) -> &'t str {
        unsafe { str::from_utf8_unchecked(self.as_bytes(raw)) }
    }

    pub fn as_string(&self, raw: &[u8]) -> String {
        bytes_to_string(self.as_bytes(raw))
    }
}

pub struct AttributeDescriptor {
    pub name: RawSubslice,
    pub value: RawSubslice,
}

pub enum TokenDescriptor<'t> {
    Character,

    Comment,

    StartTag {
        name: RawSubslice,
        attributes: &'t [AttributeDescriptor],
        self_closing: bool,
    },

    EndTag {
        name: RawSubslice,
    },

    Doctype {
        name: Option<RawSubslice>,
        public_id: Option<RawSubslice>,
        system_id: Option<RawSubslice>,
        force_quirks: bool,
    },

    Eof,
}

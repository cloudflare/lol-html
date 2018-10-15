use base::{Bytes, Range};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct AttributeView {
    pub name: Range,
    pub value: Range,
}

#[derive(Debug)]
pub struct Attribute<'b> {
    pub name: Bytes<'b>,
    pub value: Bytes<'b>,
}

#[derive(Debug)]
pub enum TokenView {
    Character,

    Comment(Range),

    StartTag {
        name: Range,
        name_hash: Option<u64>,
        attributes: Rc<RefCell<Vec<AttributeView>>>,
        self_closing: bool,
    },

    EndTag {
        name: Range,
        name_hash: Option<u64>,
    },

    Doctype {
        name: Option<Range>,
        public_id: Option<Range>,
        system_id: Option<Range>,
        force_quirks: bool,
    },

    Eof,
}

#[derive(Debug)]
pub enum Token<'b> {
    Character(Bytes<'b>),

    Comment(Bytes<'b>),

    StartTag {
        name: Bytes<'b>,
        attributes: Vec<Attribute<'b>>,
        self_closing: bool,
    },

    EndTag {
        name: Bytes<'b>,
    },

    Doctype {
        name: Option<Bytes<'b>>,
        public_id: Option<Bytes<'b>>,
        system_id: Option<Bytes<'b>>,
        force_quirks: bool,
    },

    Eof,
}

use base::{Bytes, Range};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct AttributeView {
    pub name: Range,
    pub value: Range,
}

#[derive(Debug)]
pub struct Attribute<'c> {
    pub name: Bytes<'c>,
    pub value: Bytes<'c>,
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
pub enum Token<'c> {
    Character(Bytes<'c>),

    Comment(Bytes<'c>),

    StartTag {
        name: Bytes<'c>,
        attributes: Vec<Attribute<'c>>,
        self_closing: bool,
    },

    EndTag {
        name: Bytes<'c>,
    },

    Doctype {
        name: Option<Bytes<'c>>,
        public_id: Option<Bytes<'c>>,
        system_id: Option<Bytes<'c>>,
        force_quirks: bool,
    },

    Eof,
}

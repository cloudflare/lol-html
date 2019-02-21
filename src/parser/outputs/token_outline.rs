use crate::base::{Align, Range};
use crate::parser::TextType;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct AttributeOultine {
    pub name: Range,
    pub value: Range,
    pub raw_range: Range,
}

impl Align for AttributeOultine {
    #[inline]
    fn align(&mut self, offset: usize) {
        self.name.align(offset);
        self.value.align(offset);
        self.raw_range.align(offset);
    }
}

#[derive(Debug)]
pub enum TagTokenOutline {
    StartTag {
        name: Range,
        name_hash: Option<u64>,
        attributes: Rc<RefCell<Vec<AttributeOultine>>>,
        self_closing: bool,
    },

    EndTag {
        name: Range,
        name_hash: Option<u64>,
    },
}

#[derive(Debug)]
pub enum NonTagContentTokenOutline {
    Text(TextType),
    Comment(Range),

    Doctype {
        name: Option<Range>,
        public_id: Option<Range>,
        system_id: Option<Range>,
        force_quirks: bool,
    },

    Eof,
}

impl Align for TagTokenOutline {
    #[inline]
    fn align(&mut self, offset: usize) {
        match self {
            TagTokenOutline::StartTag {
                name, attributes, ..
            } => {
                name.align(offset);
                attributes.borrow_mut().align(offset);
            }
            TagTokenOutline::EndTag { name, .. } => name.align(offset),
        }
    }
}

impl Align for NonTagContentTokenOutline {
    #[inline]
    fn align(&mut self, offset: usize) {
        match self {
            NonTagContentTokenOutline::Comment(text) => text.align(offset),
            NonTagContentTokenOutline::Doctype {
                name,
                public_id,
                system_id,
                ..
            } => {
                name.align(offset);
                public_id.align(offset);
                system_id.align(offset);
            }
            _ => (),
        }
    }
}

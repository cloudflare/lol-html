use crate::base::{Align, Range};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct AttributeView {
    pub name: Range,
    pub value: Range,
}

impl Align for AttributeView {
    fn align(&mut self, offset: usize) {
        self.name.align(offset);
        self.value.align(offset);
    }
}

// TODO create shortcuts for id and class attributes
// without necessity to iterate over attributes vector.
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

impl Align for TokenView {
    fn align(&mut self, offset: usize) {
        match self {
            TokenView::Comment(text) => text.align(offset),
            TokenView::StartTag {
                name, attributes, ..
            } => {
                name.align(offset);
                attributes.borrow_mut().align(offset);
            }
            TokenView::EndTag { name, .. } => name.align(offset),
            TokenView::Doctype {
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

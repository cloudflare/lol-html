use super::AttributeView;
use crate::base::{Bytes, Chunk};
use lazycell::LazyCell;
use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug)]
pub struct Attribute<'c> {
    pub name: Bytes<'c>,
    pub value: Bytes<'c>,
}

pub struct AttributeList<'c> {
    input: &'c Chunk<'c>,
    attribute_views: Rc<RefCell<Vec<AttributeView>>>,
    attributes: LazyCell<Vec<Attribute<'c>>>,
}

impl<'c> AttributeList<'c> {
    pub fn new(input: &'c Chunk<'c>, attribute_views: Rc<RefCell<Vec<AttributeView>>>) -> Self {
        AttributeList {
            input,
            attribute_views,
            attributes: LazyCell::new(),
        }
    }
}

impl<'c> Deref for AttributeList<'c> {
    type Target = Vec<Attribute<'c>>;

    fn deref(&self) -> &Vec<Attribute<'c>> {
        self.attributes.borrow_with(|| {
            self.attribute_views
                .borrow()
                .iter()
                .map(|a| Attribute {
                    name: self.input.slice(a.name),
                    value: self.input.slice(a.value),
                })
                .collect()
        })
    }
}

impl<'c> Debug for AttributeList<'c> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

#[derive(Debug)]
pub enum Token<'c> {
    Character(Bytes<'c>),
    Comment(Bytes<'c>),

    StartTag {
        name: Bytes<'c>,
        attributes: AttributeList<'c>,
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

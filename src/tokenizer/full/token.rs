use base::{Align, Bytes, Chunk, Range};
use lazycell::LazyCell;
use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::ops::Deref;
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
    pub fn new(input: &'c Chunk, attribute_views: &Rc<RefCell<Vec<AttributeView>>>) -> Self {
        AttributeList {
            input,
            attribute_views: Rc::clone(&attribute_views),
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
                }).collect()
        })
    }
}

impl<'c> Debug for AttributeList<'c> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

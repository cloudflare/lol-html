use base::{Alignable, Bytes, IterableChunk, Range};
use lazycell::LazyCell;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct AttributeView {
    pub name: Range,
    pub value: Range,
}

impl Alignable for AttributeView {
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

impl Alignable for TokenView {
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

#[derive(Debug)]
pub struct StartTagToken<'c> {
    input_chunk: &'c IterableChunk<'c>,
    pub name: Bytes<'c>,
    pub self_closing: bool,
    attributes_view: Rc<RefCell<Vec<AttributeView>>>,
    attributes: LazyCell<Vec<Attribute<'c>>>,
}

impl<'c> StartTagToken<'c> {
    pub fn new(
        input_chunk: &'c IterableChunk<'c>,
        name: Bytes<'c>,
        attributes_view: &Rc<RefCell<Vec<AttributeView>>>,
        self_closing: bool,
    ) -> Self {
        StartTagToken {
            input_chunk,
            name,
            attributes_view: Rc::clone(&attributes_view),
            self_closing,
            attributes: LazyCell::new(),
        }
    }

    pub fn get_attributes(&self) -> &Vec<Attribute<'c>> {
        self.attributes.borrow_with(|| {
            self.attributes_view
                .borrow()
                .iter()
                .map(|a| Attribute {
                    name: self.input_chunk.slice(a.name),
                    value: self.input_chunk.slice(a.value),
                }).collect()
        })
    }
}

#[derive(Debug)]
pub enum Token<'c> {
    Character(Bytes<'c>),
    Comment(Bytes<'c>),

    // NOTE: start tag is a special case since it contains attribute
    // collection that requires allocation, and, in order to make this
    // allocation lazy, we need intermidiate structure for this enum variant.
    StartTag(StartTagToken<'c>),

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

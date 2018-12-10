use crate::base::{Bytes, Chunk};
use crate::tokenizer::AttributeView;
use lazycell::LazyCell;
use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Getters, Debug)]
pub struct Attribute<'i> {
    #[get = "pub"]
    name: Bytes<'i>,

    #[get = "pub"]
    value: Bytes<'i>,
}

impl<'i> Attribute<'i> {
    pub fn new(name: Bytes<'i>, value: Bytes<'i>) -> Self {
        Attribute { name, value }
    }
}

pub trait AttributeList<'i>: Debug + Deref<Target = Vec<Attribute<'i>>> {}
pub type Attributes<'i> = Box<dyn AttributeList<'i, Target = Vec<Attribute<'i>>> + 'i>;

pub struct ParsedAttributeList<'i> {
    input: &'i Chunk<'i>,
    attribute_views: Rc<RefCell<Vec<AttributeView>>>,
    list: LazyCell<Vec<Attribute<'i>>>,
}

impl<'i> ParsedAttributeList<'i> {
    pub fn new(input: &'i Chunk<'i>, attribute_views: Rc<RefCell<Vec<AttributeView>>>) -> Self {
        ParsedAttributeList {
            input,
            attribute_views,
            list: LazyCell::default(),
        }
    }
}

impl<'i> Deref for ParsedAttributeList<'i> {
    type Target = Vec<Attribute<'i>>;

    fn deref(&self) -> &Vec<Attribute<'i>> {
        self.list.borrow_with(|| {
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

impl Debug for ParsedAttributeList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (&**self).fmt(f)
    }
}

impl<'i> AttributeList<'i> for ParsedAttributeList<'i> {}

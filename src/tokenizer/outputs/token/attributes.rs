use super::AttributeView;
use crate::base::{Bytes, Chunk};
use lazycell::LazyCell;
use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Getters, Debug)]
pub struct Attribute<'c> {
    #[get = "pub"]
    name: Bytes<'c>,

    #[get = "pub"]
    value: Bytes<'c>,
}

impl<'c> Attribute<'c> {
    pub fn new(name: Bytes<'c>, value: Bytes<'c>) -> Self {
        Attribute { name, value }
    }
}

pub trait AttributeList<'c>: Debug + Deref<Target = Vec<Attribute<'c>>> {}
pub type Attributes<'c> = Box<dyn AttributeList<'c, Target = Vec<Attribute<'c>>> + 'c>;

pub struct ParsedAttributeList<'c> {
    input: &'c Chunk<'c>,
    attribute_views: Rc<RefCell<Vec<AttributeView>>>,
    list: LazyCell<Vec<Attribute<'c>>>,
}

impl<'c> ParsedAttributeList<'c> {
    pub fn new(input: &'c Chunk<'c>, attribute_views: Rc<RefCell<Vec<AttributeView>>>) -> Self {
        ParsedAttributeList {
            input,
            attribute_views,
            list: LazyCell::default(),
        }
    }
}

impl<'c> Deref for ParsedAttributeList<'c> {
    type Target = Vec<Attribute<'c>>;

    fn deref(&self) -> &Vec<Attribute<'c>> {
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
        self.deref().fmt(f)
    }
}

impl<'c> AttributeList<'c> for ParsedAttributeList<'c> {}

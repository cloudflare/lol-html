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

pub struct ParsedAttributeList<'i> {
    input: &'i Chunk<'i>,
    attribute_views: Rc<RefCell<Vec<AttributeView>>>,
    items: LazyCell<Vec<Attribute<'i>>>,
}

impl<'i> ParsedAttributeList<'i> {
    pub fn new(input: &'i Chunk<'i>, attribute_views: Rc<RefCell<Vec<AttributeView>>>) -> Self {
        ParsedAttributeList {
            input,
            attribute_views,
            items: LazyCell::default(),
        }
    }

    fn items(&self) -> &Vec<Attribute<'i>> {
        self.items.borrow_with(|| {
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

pub enum Attributes<'i> {
    Parsed(ParsedAttributeList<'i>),
    Custom(Vec<Attribute<'i>>),
}

impl<'i> From<ParsedAttributeList<'i>> for Attributes<'i> {
    fn from(list: ParsedAttributeList<'i>) -> Self {
        Attributes::Parsed(list)
    }
}

impl<'i> From<Vec<Attribute<'i>>> for Attributes<'i> {
    fn from(list: Vec<Attribute<'i>>) -> Self {
        Attributes::Custom(list)
    }
}

impl<'i> Deref for Attributes<'i> {
    type Target = Vec<Attribute<'i>>;

    fn deref(&self) -> &Vec<Attribute<'i>> {
        match self {
            Attributes::Parsed(list) => list.items(),
            Attributes::Custom(list) => list,
        }
    }
}

impl Debug for Attributes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (&**self).fmt(f)
    }
}

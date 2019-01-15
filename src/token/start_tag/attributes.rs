use crate::base::{Bytes, Chunk};
use crate::lexer::AttributeOultine;
use encoding_rs::Encoding;
use lazycell::LazyCell;
use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug)]
pub struct Attribute<'i> {
    name: Bytes<'i>,
    value: Bytes<'i>,
    encoding: &'static Encoding,
}

impl<'i> Attribute<'i> {
    pub(super) fn new(name: Bytes<'i>, value: Bytes<'i>, encoding: &'static Encoding) -> Self {
        Attribute {
            name,
            value,
            encoding,
        }
    }

    #[inline]
    pub fn name(&self) -> String {
        let mut name = self.name.as_string(self.encoding);

        name.make_ascii_lowercase();

        name
    }

    #[inline]
    pub fn value(&self) -> String {
        self.value.as_string(self.encoding)
    }
}

pub struct ParsedAttributeList<'i> {
    input: &'i Chunk<'i>,
    attribute_views: Rc<RefCell<Vec<AttributeOultine>>>,
    items: LazyCell<Vec<Attribute<'i>>>,
    encoding: &'static Encoding,
}

impl<'i> ParsedAttributeList<'i> {
    pub(crate) fn new(
        input: &'i Chunk<'i>,
        attribute_views: Rc<RefCell<Vec<AttributeOultine>>>,
        encoding: &'static Encoding,
    ) -> Self {
        ParsedAttributeList {
            input,
            attribute_views,
            items: LazyCell::default(),
            encoding,
        }
    }

    fn items(&self) -> &[Attribute<'i>] {
        self.items.borrow_with(|| {
            self.attribute_views
                .borrow()
                .iter()
                .map(|a| {
                    Attribute::new(
                        self.input.slice(a.name),
                        self.input.slice(a.value),
                        self.encoding,
                    )
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
    type Target = [Attribute<'i>];

    fn deref(&self) -> &[Attribute<'i>] {
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

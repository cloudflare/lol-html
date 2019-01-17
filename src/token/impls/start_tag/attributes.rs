use crate::base::{Bytes, Chunk};
use crate::parser::AttributeOultine;
use crate::transform_stream::Serialize;
use encoding_rs::Encoding;
use failure::Error;
use lazycell::LazyCell;
use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum AttributeValidationError {
    #[fail(display = "Attribute name can't be empty.")]
    EmptyName,
    #[fail(display = "{:?} character is forbidden in the attribute name", _0)]
    ForbiddenCharacterInName(char),
    #[fail(display = "The attribute name contains a character that can't \
                      be represented in the document's character encoding.")]
    UnencodableCharacterInName,
}

#[derive(Debug, Clone)]
pub struct Attribute<'i> {
    name: Bytes<'i>,
    value: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
}

impl<'i> Attribute<'i> {
    pub(in crate::token) fn new(
        name: Bytes<'i>,
        value: Bytes<'i>,
        raw: Option<Bytes<'i>>,
        encoding: &'static Encoding,
    ) -> Self {
        Attribute {
            name,
            value,
            raw,
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
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        self.name = Attribute::name_from_str(name, self.encoding)?;
        self.raw = None;

        Ok(())
    }

    #[inline]
    pub fn value(&self) -> String {
        self.value.as_string(self.encoding)
    }

    #[inline]
    pub fn set_value(&mut self, value: &str) {
        self.value = Bytes::from_str(value, self.encoding).into_owned();
        self.raw = None;
    }

    pub(in crate::token) fn name_from_str(
        name: &str,
        encoding: &'static Encoding,
    ) -> Result<Bytes<'static>, Error> {
        if name.is_empty() {
            Err(AttributeValidationError::EmptyName.into())
        } else if let Some(ch) = name.chars().find(|&ch| match ch {
            ' ' | '\n' | '\r' | '\t' | '\x0C' | '/' | '>' | '=' => true,
            _ => false,
        }) {
            Err(AttributeValidationError::ForbiddenCharacterInName(ch).into())
        } else {
            // NOTE: if character can't be represented in the given
            // encoding then encoding_rs replaces it with a numeric
            // character reference. Character references are not
            // supported in attribute names, so we need to bail.
            match Bytes::from_str_without_replacements(name, encoding) {
                Some(name) => Ok(name.into_owned()),
                None => Err(AttributeValidationError::UnencodableCharacterInName.into()),
            }
        }
    }
}

impl Serialize for Attribute<'_> {
    #[inline]
    fn take_raw(&mut self) -> Option<Bytes<'_>> {
        self.raw.take()
    }

    #[inline]
    fn serialize_from_parts(self, handler: &mut dyn FnMut(Bytes<'_>)) {
        handler(self.name);
        handler(b"=\"".into());

        self.value.replace_ch(b'"', b"&quot;", handler);
        handler(b"\"".into());
    }
}

pub struct ParsedAttributeList<'i> {
    input: &'i Chunk<'i>,
    attribute_views: Rc<RefCell<Vec<AttributeOultine>>>,
    items: LazyCell<Vec<Attribute<'i>>>,
    encoding: &'static Encoding,
}

impl<'i> ParsedAttributeList<'i> {
    pub(in crate::token) fn new(
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
                        Some(self.input.slice(a.raw_range)),
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

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
pub enum AttributeNameError {
    #[fail(display = "Attribute name can't be empty.")]
    Empty,
    #[fail(display = "{:?} character is forbidden in the attribute name", _0)]
    ForbiddenCharacter(char),
    #[fail(display = "The attribute name contains a character that can't \
                      be represented in the document's character encoding.")]
    UnencodableCharacter,
}

pub struct Attribute<'i> {
    name: Bytes<'i>,
    value: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
}

impl<'i> Attribute<'i> {
    fn new_parsed(
        name: Bytes<'i>,
        value: Bytes<'i>,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Self {
        Attribute {
            name,
            value,
            raw: Some(raw),
            encoding,
        }
    }

    #[inline]
    fn name_from_str(name: &str, encoding: &'static Encoding) -> Result<Bytes<'static>, Error> {
        if name.is_empty() {
            Err(AttributeNameError::Empty.into())
        } else if let Some(ch) = name.chars().find(|&ch| match ch {
            ' ' | '\n' | '\r' | '\t' | '\x0C' | '/' | '>' | '=' => true,
            _ => false,
        }) {
            Err(AttributeNameError::ForbiddenCharacter(ch).into())
        } else {
            // NOTE: if character can't be represented in the given
            // encoding then encoding_rs replaces it with a numeric
            // character reference. Character references are not
            // supported in attribute names, so we need to bail.
            match Bytes::from_str_without_replacements(name, encoding) {
                Some(name) => Ok(name.into_owned()),
                None => Err(AttributeNameError::UnencodableCharacter.into()),
            }
        }
    }

    #[inline]
    fn try_from(name: &str, value: &str, encoding: &'static Encoding) -> Result<Self, Error> {
        Ok(Attribute {
            name: Attribute::name_from_str(name, encoding)?,
            value: Bytes::from_str(value, encoding).into_owned(),
            raw: None,
            encoding,
        })
    }

    // NOTE: not a trait implementation due to the `Borrow` constraint for
    // the `Owned` associated type.
    // See: https://github.com/rust-lang/rust/issues/44950
    #[inline]
    pub fn to_owned(&self) -> Attribute<'static> {
        Attribute {
            name: self.name.to_owned(),
            value: self.value.to_owned(),
            raw: Bytes::opt_to_owned(&self.raw),
            encoding: self.encoding,
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

    #[inline]
    fn set_value(&mut self, value: &str) {
        self.value = Bytes::from_str(value, self.encoding).into_owned();
        self.raw = None;
    }
}

impl Serialize for Attribute<'_> {
    #[inline]
    fn raw(&self) -> Option<&Bytes<'_>> {
        self.raw.as_ref()
    }

    #[inline]
    fn serialize_from_parts(&self, handler: &mut dyn FnMut(&Bytes<'_>)) {
        handler(&self.name);
        handler(&b"=\"".into());

        self.value.replace_byte((b'"', b"&quot;"), handler);
        handler(&b"\"".into());
    }
}

impl Debug for Attribute<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Attribute")
            .field("name", &self.name())
            .field("value", &self.value())
            .finish()
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

    fn init_items(&self) -> Vec<Attribute<'i>> {
        self.attribute_views
            .borrow()
            .iter()
            .map(|a| {
                Attribute::new_parsed(
                    self.input.slice(a.name),
                    self.input.slice(a.value),
                    self.input.slice(a.raw_range),
                    self.encoding,
                )
            })
            .collect()
    }

    #[inline]
    fn as_mut_vec(&mut self) -> &mut Vec<Attribute<'i>> {
        // NOTE: we can't use borrow_mut_with here as we'll need
        // because `self` is a mutable reference and we'll have
        // two mutable references by passing it to the initializer
        // closure.
        if !self.items.filled() {
            self.items
                .fill(self.init_items())
                .expect("Cell should be empty at this point");
        }

        self.items
            .borrow_mut()
            .expect("Items should be initialized")
    }
}

impl<'i> Deref for ParsedAttributeList<'i> {
    type Target = [Attribute<'i>];

    #[inline]
    fn deref(&self) -> &[Attribute<'i>] {
        self.items.borrow_with(|| self.init_items())
    }
}

pub enum Attributes<'i> {
    Parsed(ParsedAttributeList<'i>),
    Custom(Vec<Attribute<'i>>),
}

impl<'i> Attributes<'i> {
    pub(super) fn try_from(
        items: &[(&str, &str)],
        encoding: &'static Encoding,
    ) -> Result<Self, Error> {
        Ok(Attributes::Custom(
            items
                .iter()
                .map(|(name, value)| Attribute::try_from(name, value, encoding))
                .collect::<Result<_, _>>()?,
        ))
    }

    #[inline]
    fn as_mut_vec(&mut self) -> &mut Vec<Attribute<'i>> {
        match self {
            Attributes::Parsed(l) => l.as_mut_vec(),
            Attributes::Custom(l) => l,
        }
    }

    pub fn set_attribute(
        &mut self,
        name: &str,
        value: &str,
        encoding: &'static Encoding,
    ) -> Result<(), Error> {
        let name = name.to_ascii_lowercase();
        let items = self.as_mut_vec();

        match items.iter_mut().find(|attr| attr.name() == name.as_str()) {
            Some(attr) => attr.set_value(value),
            None => {
                items.push(Attribute::try_from(&name, value, encoding)?);
            }
        }

        Ok(())
    }

    pub fn remove_attribute(&mut self, name: &str) -> bool {
        let name = name.to_ascii_lowercase();
        let items = self.as_mut_vec();
        let mut i = 0;

        while i < items.len() {
            if items[i].name() == name.as_str() {
                items.remove(i);
                return true;
            }

            i += 1;
        }

        false
    }

    // NOTE: not a trait implementation due to the `Borrow` constraint for
    // the `Owned` associated type.
    // See: https://github.com/rust-lang/rust/issues/44950
    #[inline]
    pub fn to_owned(&self) -> Attributes<'static> {
        Attributes::Custom(self.iter().map(|a| a.to_owned()).collect())
    }
}

impl Serialize for Attributes<'_> {
    #[inline]
    fn raw(&self) -> Option<&Bytes<'_>> {
        None
    }

    #[inline]
    fn serialize_from_parts(&self, handler: &mut dyn FnMut(&Bytes<'_>)) {
        if !self.is_empty() {
            let last = self.len() - 1;

            for (idx, attr) in self.iter().enumerate() {
                attr.to_bytes(handler);

                if idx != last {
                    handler(&b" ".into());
                }
            }
        }
    }
}

impl<'i> From<ParsedAttributeList<'i>> for Attributes<'i> {
    fn from(list: ParsedAttributeList<'i>) -> Self {
        Attributes::Parsed(list)
    }
}

impl<'i> Deref for Attributes<'i> {
    type Target = [Attribute<'i>];

    #[inline]
    fn deref(&self) -> &[Attribute<'i>] {
        match self {
            Attributes::Parsed(l) => l,
            Attributes::Custom(l) => l,
        }
    }
}

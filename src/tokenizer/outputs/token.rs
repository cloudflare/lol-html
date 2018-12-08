use super::AttributeView;
use crate::base::{Bytes, Chunk};
use lazycell::LazyCell;
use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Getters, Debug)]
pub struct CharacterToken<'c> {
    #[get = "pub"]
    text: Bytes<'c>,
}

#[derive(Getters, Debug)]
pub struct CommentToken<'c> {
    #[get = "pub"]
    text: Bytes<'c>,
}

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

struct ParsedAttributesData<'c> {
    input: &'c Chunk<'c>,
    attribute_views: Rc<RefCell<Vec<AttributeView>>>,
}

#[derive(Default)]
pub struct AttributeList<'c> {
    parsed_attributes_data: Option<ParsedAttributesData<'c>>,
    attributes: LazyCell<Vec<Attribute<'c>>>,
}

impl<'c> AttributeList<'c> {
    pub fn new(input: &'c Chunk<'c>, attribute_views: Rc<RefCell<Vec<AttributeView>>>) -> Self {
        AttributeList {
            parsed_attributes_data: Some(ParsedAttributesData {
                input,
                attribute_views,
            }),
            attributes: LazyCell::default(),
        }
    }

    fn init(&self) -> Vec<Attribute<'c>> {
        match self.parsed_attributes_data {
            Some(ParsedAttributesData {
                ref input,
                ref attribute_views,
            }) => attribute_views
                .borrow()
                .iter()
                .map(|a| Attribute {
                    name: input.slice(a.name),
                    value: input.slice(a.value),
                })
                .collect(),
            None => Vec::new(),
        }
    }
}

impl<'c> Deref for AttributeList<'c> {
    type Target = Vec<Attribute<'c>>;

    fn deref(&self) -> &Vec<Attribute<'c>> {
        self.attributes.borrow_with(|| self.init())
    }
}

impl Debug for AttributeList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

#[derive(Getters, Debug)]
pub struct StartTagToken<'c> {
    #[get = "pub"]
    name: Bytes<'c>,

    #[get = "pub"]
    attributes: AttributeList<'c>,

    self_closing: bool,
}

impl<'c> StartTagToken<'c> {
    #[inline]
    pub fn self_closing(&self) -> bool {
        self.self_closing
    }
}

#[derive(Getters, Debug)]
pub struct EndTagToken<'c> {
    #[get = "pub"]
    name: Bytes<'c>,
}

#[derive(Debug)]
pub struct DoctypeToken<'c> {
    name: Option<Bytes<'c>>,
    public_id: Option<Bytes<'c>>,
    system_id: Option<Bytes<'c>>,
    force_quirks: bool,
}

impl<'c> DoctypeToken<'c> {
    #[inline]
    pub fn name(&self) -> Option<&Bytes<'c>> {
        self.name.as_ref()
    }

    #[inline]
    pub fn public_id(&self) -> Option<&Bytes<'c>> {
        self.public_id.as_ref()
    }

    #[inline]
    pub fn system_id(&self) -> Option<&Bytes<'c>> {
        self.system_id.as_ref()
    }

    #[inline]
    pub fn force_quirks(&self) -> bool {
        self.force_quirks
    }
}

#[derive(Debug)]
pub enum Token<'c> {
    Character(CharacterToken<'c>),
    Comment(CommentToken<'c>),
    StartTag(StartTagToken<'c>),
    EndTag(EndTagToken<'c>),
    Doctype(DoctypeToken<'c>),
    Eof,
}

impl<'c> Token<'c> {
    pub fn new_character(text: Bytes<'c>) -> Self {
        Token::Character(CharacterToken { text })
    }

    pub fn new_comment(text: Bytes<'c>) -> Self {
        Token::Comment(CommentToken { text })
    }

    pub fn new_start_tag(
        name: Bytes<'c>,
        attributes: AttributeList<'c>,
        self_closing: bool,
    ) -> Self {
        Token::StartTag(StartTagToken {
            name,
            attributes,
            self_closing,
        })
    }

    pub fn new_end_tag(name: Bytes<'c>) -> Self {
        Token::EndTag(EndTagToken { name })
    }

    pub fn new_doctype(
        name: Option<Bytes<'c>>,
        public_id: Option<Bytes<'c>>,
        system_id: Option<Bytes<'c>>,
        force_quirks: bool,
    ) -> Self {
        Token::Doctype(DoctypeToken {
            name,
            public_id,
            system_id,
            force_quirks,
        })
    }
}

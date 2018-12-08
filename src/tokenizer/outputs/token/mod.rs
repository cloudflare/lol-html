mod attributes;

pub use self::attributes::*;
use super::AttributeView;
use crate::base::Bytes;

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
pub struct StartTagToken<'c> {
    #[get = "pub"]
    name: Bytes<'c>,

    #[get = "pub"]
    attributes: Attributes<'c>,

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
        attributes: ParsedAttributeList<'c>,
        self_closing: bool,
    ) -> Self {
        Token::StartTag(StartTagToken {
            name,
            attributes: Box::new(attributes),
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

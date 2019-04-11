use super::SelectorError;
use crate::html::Namespace;
use cssparser::{Parser as CssParser, ParserInput, ToCss};
use selectors::parser::{
    NonTSPseudoClass, Parser, PseudoElement, SelectorImpl, SelectorList, SelectorParseErrorKind,
};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct SelectorImplDescriptor;

impl SelectorImpl for SelectorImplDescriptor {
    type AttrValue = String;
    type Identifier = String;
    type ClassName = String;
    type LocalName = String;
    type NamespacePrefix = String;
    type NamespaceUrl = Namespace;
    type BorrowedNamespaceUrl = Namespace;
    type BorrowedLocalName = String;

    type NonTSPseudoClass = NonTSPseudoClassStub;
    type PseudoElement = PseudoElementStub;

    type ExtraMatchingData = ();
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum PseudoElementStub {}

impl ToCss for PseudoElementStub {
    fn to_css<W: fmt::Write>(&self, _dest: &mut W) -> fmt::Result {
        match *self {}
    }
}

impl PseudoElement for PseudoElementStub {
    type Impl = SelectorImplDescriptor;
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum NonTSPseudoClassStub {}

impl NonTSPseudoClass for NonTSPseudoClassStub {
    type Impl = SelectorImplDescriptor;

    fn is_active_or_hover(&self) -> bool {
        match *self {}
    }
}

impl ToCss for NonTSPseudoClassStub {
    fn to_css<W: fmt::Write>(&self, _dest: &mut W) -> fmt::Result {
        match *self {}
    }
}

#[allow(dead_code)]
pub struct ParserImplDescriptor;

impl<'i> Parser<'i> for ParserImplDescriptor {
    type Impl = SelectorImplDescriptor;
    type Error = SelectorParseErrorKind<'i>;
}

#[inline]
pub fn parse_selector(
    selector: &str,
) -> Result<SelectorList<SelectorImplDescriptor>, SelectorError> {
    let mut input = ParserInput::new(selector);
    let mut css_parser = CssParser::new(&mut input);

    SelectorList::parse(&ParserImplDescriptor, &mut css_parser).map_err(SelectorError::from)
}

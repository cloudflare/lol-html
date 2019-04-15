use super::SelectorError;
use crate::html::Namespace;
use cssparser::{Parser as CssParser, ParserInput, ToCss};
use selectors::parser::{
    Combinator, Component, NonTSPseudoClass, Parser, PseudoElement, SelectorImpl, SelectorList,
    SelectorParseErrorKind,
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
pub struct SelectorsParser;

impl SelectorsParser {
    #[inline]
    fn validate_components(
        selector_list: SelectorList<SelectorImplDescriptor>,
    ) -> Result<SelectorList<SelectorImplDescriptor>, SelectorError> {
        for selector in selector_list.0.iter() {
            for component in selector.iter_raw_match_order() {
                // NOTE: always use explicit variants in this match, so we
                // get compile-time error if new component types were added to
                // the parser.
                #[deny(clippy::wildcard_enum_match_arm)]
                match component {
                    Component::Combinator(combinator) => match combinator {
                        // Supported
                        Combinator::Child | Combinator::Descendant => (),

                        // Unsupported
                        Combinator::NextSibling => {
                            return Err(SelectorError::UnsupportedCombinator('+'));
                        }
                        Combinator::LaterSibling => {
                            return Err(SelectorError::UnsupportedCombinator('~'));
                        }
                        Combinator::PseudoElement | Combinator::SlotAssignment => unreachable!(
                            "Pseudo element combinators should be filtered out at this point"
                        ),
                    },

                    // Supported
                    Component::LocalName(_)
                    | Component::ExplicitUniversalType
                    | Component::ExplicitAnyNamespace
                    | Component::ExplicitNoNamespace
                    | Component::ID(_)
                    | Component::Class(_)
                    | Component::AttributeInNoNamespaceExists { .. }
                    | Component::AttributeInNoNamespace { .. }
                    | Component::Negation(_) => (),

                    // Unsupported
                    Component::Empty
                    | Component::FirstChild
                    | Component::FirstOfType
                    | Component::Host(_)
                    | Component::LastChild
                    | Component::LastOfType
                    | Component::NthChild(_, _)
                    | Component::NthLastChild(_, _)
                    | Component::NthLastOfType(_, _)
                    | Component::NthOfType(_, _)
                    | Component::OnlyChild
                    | Component::OnlyOfType
                    | Component::Root
                    | Component::Scope
                    | Component::PseudoElement(_)
                    | Component::NonTSPseudoClass(_)
                    | Component::Slotted(_) => {
                        return Err(SelectorError::UnsupportedPseudoClassOrElement)
                    }

                    Component::DefaultNamespace(_)
                    | Component::Namespace(_, _)
                    | Component::AttributeOther(_) => {
                        return Err(SelectorError::NamespacedSelector)
                    }
                }
            }
        }

        Ok(selector_list)
    }

    #[inline]
    pub fn parse(selector: &str) -> Result<SelectorList<SelectorImplDescriptor>, SelectorError> {
        let mut input = ParserInput::new(selector);
        let mut css_parser = CssParser::new(&mut input);

        SelectorList::parse(&Self, &mut css_parser)
            .map_err(SelectorError::from)
            .and_then(Self::validate_components)
    }
}

impl<'i> Parser<'i> for SelectorsParser {
    type Impl = SelectorImplDescriptor;
    type Error = SelectorParseErrorKind<'i>;
}

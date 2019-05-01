use cssparser::{BasicParseErrorKind, ParseErrorKind};
use selectors::parser::{SelectorParseError, SelectorParseErrorKind};

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum SelectorError {
    #[fail(display = "Unexpected token in selector.")]
    UnexpectedToken,
    #[fail(display = "Unexpected end of selector.")]
    UnexpectedEnd,
    #[fail(display = "Missing attribute name in attribute selector.")]
    MissingAttributeName,
    #[fail(display = "The selector is empty.")]
    EmptySelector,
    #[fail(display = "Dangling combinator in selector.")]
    DanglingCombinator,
    #[fail(display = "Unexpected token in the attribute selector.")]
    UnexpectedTokenInAttribute,
    #[fail(display = "Unsupported pseudo-class or pseudo-element in selector.")]
    UnsupportedPseudoClassOrElement,
    #[fail(display = "Nested negation in selector.")]
    NestedNegation,
    #[fail(display = "Selectors with explicit namespaces are not supported.")]
    NamespacedSelector,
    #[fail(display = "Invalid or unescaped class name in selector.")]
    InvalidClassName,
    #[fail(display = "Empty negation in selector.")]
    EmptyNegation,
    #[fail(display = "Unsupported combinator `{}` in selector.", _0)]
    UnsupportedCombinator(char),
    #[fail(display = "Unsupported syntax in selector.")]
    UnsupportedSyntax,
}

impl From<SelectorParseError<'_>> for SelectorError {
    fn from(err: SelectorParseError) -> Self {
        // NOTE: always use explicit variants in this match, so we
        // get compile-time error if new error types were added to
        // the parser.
        #[deny(clippy::wildcard_enum_match_arm)]
        match err.kind {
            ParseErrorKind::Basic(err) => match err {
                BasicParseErrorKind::UnexpectedToken(_) => SelectorError::UnexpectedToken,
                BasicParseErrorKind::EndOfInput => SelectorError::UnexpectedEnd,
                BasicParseErrorKind::AtRuleBodyInvalid
                | BasicParseErrorKind::AtRuleInvalid(_)
                | BasicParseErrorKind::QualifiedRuleInvalid => SelectorError::UnsupportedSyntax,
            },
            ParseErrorKind::Custom(err) => match err {
                SelectorParseErrorKind::NoQualifiedNameInAttributeSelector(_) => {
                    SelectorError::MissingAttributeName
                }
                SelectorParseErrorKind::EmptySelector => SelectorError::EmptySelector,
                SelectorParseErrorKind::DanglingCombinator => SelectorError::DanglingCombinator,
                SelectorParseErrorKind::UnsupportedPseudoClassOrElement(_)
                | SelectorParseErrorKind::PseudoElementInComplexSelector
                | SelectorParseErrorKind::NonPseudoElementAfterSlotted
                | SelectorParseErrorKind::InvalidPseudoElementAfterSlotted
                | SelectorParseErrorKind::PseudoElementExpectedColon(_)
                | SelectorParseErrorKind::PseudoElementExpectedIdent(_)
                | SelectorParseErrorKind::NoIdentForPseudo(_)
                // NOTE: according to the parser code this error occures only during
                // the parsing of vendor-specific pseudo-classes.
                | SelectorParseErrorKind::NonCompoundSelector
                // NOTE: according to the parser code this error occures only during
                // the parsing of the :slotted() pseudo-class.
                | SelectorParseErrorKind::NonSimpleSelectorInNegation => {
                    SelectorError::UnsupportedPseudoClassOrElement
                }
                // NOTE: this is currently the only case in the parser code
                // that triggers this error.
                SelectorParseErrorKind::UnexpectedIdent(_) => SelectorError::NestedNegation,
                SelectorParseErrorKind::ExpectedNamespace(_) => SelectorError::NamespacedSelector,
                SelectorParseErrorKind::ExplicitNamespaceUnexpectedToken(_) => {
                    SelectorError::UnexpectedToken
                }
                SelectorParseErrorKind::UnexpectedTokenInAttributeSelector(_)
                | SelectorParseErrorKind::ExpectedBarInAttr(_)
                | SelectorParseErrorKind::BadValueInAttr(_)
                | SelectorParseErrorKind::InvalidQualNameInAttr(_) => {
                    SelectorError::UnexpectedTokenInAttribute
                }
                SelectorParseErrorKind::ClassNeedsIdent(_) => SelectorError::InvalidClassName,
                SelectorParseErrorKind::EmptyNegation => SelectorError::EmptyNegation,
            },
        }
    }
}

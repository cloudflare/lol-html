use cssparser::{BasicParseErrorKind, ParseErrorKind};
use selectors::parser::{SelectorParseError, SelectorParseErrorKind};

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum SelectorError {
    #[fail(display = "Unexpected token in the selector.")]
    UnexpectedToken,
    #[fail(display = "Unexpected end of the selector.")]
    UnexpectedEnd,
    #[fail(display = "Missing attribute name in the attribute selector.")]
    MissingAttributeName,
    #[fail(display = "The selector is empty.")]
    EmptySelector,
    #[fail(display = "Dangling combinator in the selector.")]
    DanglingCombinator,
    #[fail(display = "Unexpected token in the attribute selector.")]
    UnexpectedTokenInAttributeSelector,
    #[fail(display = "Pseudo classes and elements are unsupported in selectors.")]
    UnsupportedPseudoClassOrElement,
    #[fail(display = "Unexpected identifier in the selector.")]
    UnexpectedIdent,
    #[fail(display = "Selector with explicit namespaces are not supported.")]
    NamespacedSelector,
    #[fail(display = "Unexpected token in the attribute selector.")]
    UnexpectedTokenInAttr,
    #[fail(display = "Invalid or unescaped class name in the selector.")]
    InvalidClassName,
    #[fail(display = "Empty negation in the selector.")]
    EmptyNegation,
    #[fail(display = "Unsupported combinator `{}` in the selector.", _0)]
    UnsupportedCombinator(char),
    #[fail(display = "Unsupported namespaced attribute selector.")]
    UnsupportedNamespacedAttributeSelector,
    #[fail(display = "Unsupported syntax in the selector.")]
    UnsupportedSyntax,
}

impl From<SelectorParseError<'_>> for SelectorError {
    fn from(err: SelectorParseError<'_>) -> Self {
        match err.kind {
            ParseErrorKind::Basic(err) => match err {
                BasicParseErrorKind::UnexpectedToken(_) => SelectorError::UnexpectedToken,
                BasicParseErrorKind::EndOfInput => SelectorError::UnexpectedEnd,
                _ => SelectorError::UnsupportedSyntax,
            },
            ParseErrorKind::Custom(err) => match err {
                SelectorParseErrorKind::NoQualifiedNameInAttributeSelector(_) => {
                    SelectorError::MissingAttributeName
                }
                SelectorParseErrorKind::EmptySelector => SelectorError::EmptySelector,
                SelectorParseErrorKind::DanglingCombinator => SelectorError::DanglingCombinator,
                SelectorParseErrorKind::UnexpectedTokenInAttributeSelector(_) => {
                    SelectorError::UnexpectedTokenInAttributeSelector
                }
                SelectorParseErrorKind::UnsupportedPseudoClassOrElement(_) => {
                    SelectorError::UnsupportedPseudoClassOrElement
                }
                SelectorParseErrorKind::UnexpectedIdent(_) => SelectorError::UnexpectedIdent,
                SelectorParseErrorKind::ExpectedNamespace(_) => SelectorError::NamespacedSelector,
                SelectorParseErrorKind::ExplicitNamespaceUnexpectedToken(_) => {
                    SelectorError::UnexpectedToken
                }
                SelectorParseErrorKind::ExpectedBarInAttr(_)
                | SelectorParseErrorKind::BadValueInAttr(_)
                | SelectorParseErrorKind::InvalidQualNameInAttr(_) => {
                    SelectorError::UnexpectedTokenInAttr
                }
                SelectorParseErrorKind::ClassNeedsIdent(_) => SelectorError::InvalidClassName,
                SelectorParseErrorKind::EmptyNegation => SelectorError::EmptyNegation,
                _ => SelectorError::UnsupportedSyntax,
            },
        }
    }
}

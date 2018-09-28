mod text_parsing_ambiguity;
mod text_parsing_mode;

use self::text_parsing_ambiguity::TextParsingAmbiguityTracker;
pub use self::text_parsing_mode::*;
use lex_unit::{Attribute, Token};
use tag_name::TagName;
use tokenizer::TokenizerErrorKind;

const DEFAULT_NS_STACK_CAPACITY: usize = 256;

#[derive(Copy, Clone, Eq, PartialEq)]
enum Namespace {
    Html,
    Svg,
    MathML,
}

#[derive(Copy, Clone)]
pub enum StartTagTokenRequestReason {
    ForeignContentExitCheck,
    IntegrationPointCheck,
}

pub enum TokenizerAdjustment {
    SwitchTextParsingMode(TextParsingMode),
    SetAllowCdata(bool),
}

pub enum TreeBuilderFeedback {
    Adjust(TokenizerAdjustment),
    RequestStartTagToken(StartTagTokenRequestReason),
    RequestEndTagToken,
    RequestSelfClosingFlag,
    None,
}

impl TreeBuilderFeedback {
    pub fn set_allow_cdata(allow_cdata: bool) -> Self {
        TreeBuilderFeedback::Adjust(TokenizerAdjustment::SetAllowCdata(allow_cdata))
    }

    pub fn switch_text_parsing_mode(mode: TextParsingMode) -> Self {
        TreeBuilderFeedback::Adjust(TokenizerAdjustment::SwitchTextParsingMode(mode))
    }
}

#[inline]
fn eq_case_ins(actual: &[u8], expected: &[u8]) -> bool {
    if actual.len() != expected.len() {
        return false;
    }

    for i in 0..actual.len() {
        if actual[i].to_ascii_lowercase() != expected[i] {
            return false;
        }
    }

    true
}

macro_rules! tag_is_one_of {
    ($tag_name_hash:expr, [$($tag:ident),+]) => {
        $($tag_name_hash == TagName::$tag)||+
    };
}

#[inline]
fn get_text_parsing_mode_adjustment(tag_name_hash: u64) -> TreeBuilderFeedback {
    if tag_is_one_of!(tag_name_hash, [Textarea, Title]) {
        TreeBuilderFeedback::switch_text_parsing_mode(TextParsingMode::RCData)
    } else if tag_name_hash == TagName::Plaintext {
        TreeBuilderFeedback::switch_text_parsing_mode(TextParsingMode::PlainText)
    } else if tag_name_hash == TagName::Script {
        TreeBuilderFeedback::switch_text_parsing_mode(TextParsingMode::ScriptData)
    } else if tag_is_one_of!(
        tag_name_hash,
        [Style, Iframe, Xmp, Noembed, Noframes, Noscript]
    ) {
        TreeBuilderFeedback::switch_text_parsing_mode(TextParsingMode::RawText)
    } else {
        TreeBuilderFeedback::None
    }
}

fn causes_foreign_content_exit(tag_name_hash: u64) -> bool {
    tag_is_one_of!(
        tag_name_hash,
        [
            B, Big, Blockquote, Body, Br, Center, Code, Dd, Div, Dl, Dt, Em, Embed, H1, H2, H3, H4,
            H5, H6, Head, Hr, I, Img, Li, Listing, Menu, Meta, Nobr, Ol, P, Pre, Ruby, S, Small,
            Span, Strong, Strike, Sub, Sup, Table, Tt, U, Ul, Var
        ]
    )
}

fn is_text_integration_point_in_math_ml(tag_name_hash: u64) -> bool {
    tag_is_one_of!(tag_name_hash, [Mi, Mo, Mn, Ms, Mtext])
}

fn is_html_integration_point_in_svg(tag_name_hash: u64) -> bool {
    tag_is_one_of!(tag_name_hash, [Desc, Title, ForeignObject])
}

// TODO limit ns stack
pub struct TreeBuilderSimulator {
    ns_stack: Vec<Namespace>,
    current_ns: Namespace,
    text_parsing_ambiguity_tracker: TextParsingAmbiguityTracker,
}

impl Default for TreeBuilderSimulator {
    fn default() -> Self {
        let mut simulator = TreeBuilderSimulator {
            ns_stack: Vec::with_capacity(DEFAULT_NS_STACK_CAPACITY),
            current_ns: Namespace::Html,
            text_parsing_ambiguity_tracker: TextParsingAmbiguityTracker::default(),
        };

        simulator.ns_stack.push(Namespace::Html);

        simulator
    }
}

impl TreeBuilderSimulator {
    fn enter_ns(&mut self, ns: Namespace) -> TreeBuilderFeedback {
        self.ns_stack.push(ns);
        self.current_ns = ns;
        TreeBuilderFeedback::set_allow_cdata(ns != Namespace::Html)
    }

    fn leave_ns(&mut self) -> TreeBuilderFeedback {
        self.ns_stack.pop();

        self.current_ns = *self
            .ns_stack
            .last()
            .expect("Namespace stack should always have at least one item");

        TreeBuilderFeedback::set_allow_cdata(self.current_ns != Namespace::Html)
    }

    fn is_integration_point_enter(&self, tag_name_hash: u64) -> bool {
        self.current_ns == Namespace::Svg && is_html_integration_point_in_svg(tag_name_hash)
            || self.current_ns == Namespace::MathML
                && is_text_integration_point_in_math_ml(tag_name_hash)
    }

    fn check_integration_point_exit(&mut self, tag_name_hash: Option<u64>) -> TreeBuilderFeedback {
        let ns_stack_len = self.ns_stack.len();

        if ns_stack_len < 2 {
            return TreeBuilderFeedback::None;
        }

        let prev_ns = self.ns_stack[ns_stack_len - 2];

        match tag_name_hash {
            Some(t)
                if prev_ns == Namespace::MathML && is_text_integration_point_in_math_ml(t)
                    || prev_ns == Namespace::Svg && is_html_integration_point_in_svg(t) =>
            {
                self.leave_ns()
            }
            // NOTE: <annotation-xml> case
            None if prev_ns == Namespace::MathML => TreeBuilderFeedback::RequestEndTagToken,
            _ => TreeBuilderFeedback::None,
        }
    }

    fn get_feedback_for_start_tag_in_foreign_content(
        &mut self,
        tag_name_hash: Option<u64>,
    ) -> TreeBuilderFeedback {
        match tag_name_hash {
            Some(t) if causes_foreign_content_exit(t) => self.leave_ns(),
            // NOTE: <font> tag special case requires attributes
            // to decide on foreign context exit
            Some(t) if t == TagName::Font => TreeBuilderFeedback::RequestStartTagToken(
                StartTagTokenRequestReason::ForeignContentExitCheck,
            ),
            Some(t) if self.is_integration_point_enter(t) => {
                TreeBuilderFeedback::RequestSelfClosingFlag
            }
            // NOTE: integration point check <annotation-xml> case
            None if self.current_ns == Namespace::MathML => {
                TreeBuilderFeedback::RequestStartTagToken(
                    StartTagTokenRequestReason::IntegrationPointCheck,
                )
            }
            _ => TreeBuilderFeedback::None,
        }
    }

    pub fn get_feedback_for_start_tag_name(
        &mut self,
        tag_name_hash: Option<u64>,
    ) -> Result<TreeBuilderFeedback, TokenizerErrorKind> {
        self.text_parsing_ambiguity_tracker
            .track_start_tag(tag_name_hash)?;

        Ok(match tag_name_hash {
            Some(t) if t == TagName::Svg => self.enter_ns(Namespace::Svg),
            Some(t) if t == TagName::Math => self.enter_ns(Namespace::MathML),
            Some(t) if self.current_ns == Namespace::Html => get_text_parsing_mode_adjustment(t),
            _ if self.current_ns != Namespace::Html => {
                self.get_feedback_for_start_tag_in_foreign_content(tag_name_hash)
            }
            _ => TreeBuilderFeedback::None,
        })
    }

    pub fn get_feedback_for_end_tag_name(
        &mut self,
        tag_name_hash: Option<u64>,
    ) -> TreeBuilderFeedback {
        self.text_parsing_ambiguity_tracker
            .track_end_tag(tag_name_hash);

        match tag_name_hash {
            Some(t) if self.current_ns == Namespace::Svg && t == TagName::Svg => self.leave_ns(),
            Some(t) if self.current_ns == Namespace::MathML && t == TagName::Math => {
                self.leave_ns()
            }
            _ if self.current_ns == Namespace::Html => {
                self.check_integration_point_exit(tag_name_hash)
            }
            _ => TreeBuilderFeedback::None,
        }
    }

    pub fn fulfill_self_closing_flag_request(&mut self, self_closing: bool) -> TreeBuilderFeedback {
        // NOTE: we request self-closing flag only for HTML integration point check.
        if self_closing {
            TreeBuilderFeedback::None
        } else {
            self.enter_ns(Namespace::Html)
        }
    }

    pub fn fulfill_end_tag_token_request(&mut self, token: &Token) -> TreeBuilderFeedback {
        match token {
            Token::EndTag { ref name, .. } => {
                // NOTE: we request end tag token only when we
                // need attribute for `<annotation-xml>` tag in HTML
                // integration point in MathMl.
                if eq_case_ins(name, b"annotation-xml") {
                    self.leave_ns()
                } else {
                    TreeBuilderFeedback::None
                }
            }
            _ => unreachable!("Token should be an end tag at this point"),
        }
    }

    pub fn fulfill_start_tag_token_request(
        &mut self,
        token: &Token,
        request_reason: StartTagTokenRequestReason,
    ) -> TreeBuilderFeedback {
        match token {
            Token::StartTag {
                ref name,
                ref attributes,
                self_closing,
            } => match request_reason {
                StartTagTokenRequestReason::ForeignContentExitCheck => {
                    // NOTE: for foreign content exit we request token only if
                    // we saw <font> tag and we need to check its attributes.
                    for Attribute { ref name, .. } in attributes {
                        if eq_case_ins(name, b"color")
                            || eq_case_ins(name, b"size")
                            || eq_case_ins(name, b"face")
                        {
                            return self.leave_ns();
                        }
                    }
                }
                StartTagTokenRequestReason::IntegrationPointCheck => {
                    if !self_closing && eq_case_ins(name, b"annotation-xml") {
                        for Attribute {
                            ref name,
                            ref value,
                        } in attributes
                        {
                            if eq_case_ins(name, b"encoding")
                                && (eq_case_ins(value, b"text/html")
                                    || eq_case_ins(value, b"application/xhtml+xml"))
                            {
                                return self.enter_ns(Namespace::Html);
                            }
                        }
                    }
                }
            },
            _ => unreachable!("Token should be a start tag at this point"),
        }

        TreeBuilderFeedback::None
    }
}

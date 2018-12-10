//! HTML tokenizer has 6 different state machines for text parsing
//! purposes in different contexts. Switch between these state machines
//! usually performed by the tree construction stage depending on the
//! state of the stack of open elements (HTML is a context-sensitive grammar).
//!
//! Luckily, in the majority of cases this tree construction stage feedback
//! can be simulated without the stack of open elements and comlicated rules
//! required to maintain its state.
//!
//! This module implements such feedback simulation. However, there are few
//! cases where we can't unambiguously determine parsing context and prefer
//! to bail out from the tokenization in such a case
//! (see `AmbiguityGuard` for the details).

use crate::base::Bytes;
use crate::tokenizer::outputs::{LexUnit, TokenView};
use crate::tokenizer::{TagName, TextParsingMode};

const DEFAULT_NS_STACK_CAPACITY: usize = 256;

#[derive(Copy, Clone, Eq, PartialEq)]
enum Namespace {
    Html,
    Svg,
    MathML,
}

#[must_use]
#[derive(Copy, Clone)]
pub enum TreeBuilderFeedback {
    SwitchTextParsingMode(TextParsingMode),
    SetAllowCdata(bool),
    RequestLexUnit(fn(&mut TreeBuilderSimulator, &LexUnit<'_>) -> TreeBuilderFeedback),
    None,
}

#[inline]
fn eq_case_insensitive(actual: &Bytes<'_>, expected: &[u8]) -> bool {
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

#[inline]
fn get_text_parsing_mode_adjustment(tag_name_hash: u64) -> TreeBuilderFeedback {
    if tag_is_one_of!(tag_name_hash, [Textarea, Title]) {
        TreeBuilderFeedback::SwitchTextParsingMode(TextParsingMode::RCData)
    } else if tag_name_hash == TagName::Plaintext {
        TreeBuilderFeedback::SwitchTextParsingMode(TextParsingMode::PlainText)
    } else if tag_name_hash == TagName::Script {
        TreeBuilderFeedback::SwitchTextParsingMode(TextParsingMode::ScriptData)
    } else if tag_is_one_of!(
        tag_name_hash,
        [Style, Iframe, Xmp, Noembed, Noframes, Noscript]
    ) {
        TreeBuilderFeedback::SwitchTextParsingMode(TextParsingMode::RawText)
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

macro_rules! expect_token_view {
    ($lex_unit: ident) => {
        *$lex_unit
            .token_view()
            .expect("There should be a token view at this point")
    };
}

// TODO limit ns stack
pub struct TreeBuilderSimulator {
    ns_stack: Vec<Namespace>,
    current_ns: Namespace,
}

impl Default for TreeBuilderSimulator {
    fn default() -> Self {
        let mut simulator = TreeBuilderSimulator {
            ns_stack: Vec::with_capacity(DEFAULT_NS_STACK_CAPACITY),
            current_ns: Namespace::Html,
        };

        simulator.ns_stack.push(Namespace::Html);

        simulator
    }
}

impl TreeBuilderSimulator {
    pub fn get_feedback_for_start_tag_name(
        &mut self,
        tag_name_hash: Option<u64>,
    ) -> TreeBuilderFeedback {
        match tag_name_hash {
            Some(t) if t == TagName::Svg => self.enter_ns(Namespace::Svg),
            Some(t) if t == TagName::Math => self.enter_ns(Namespace::MathML),
            Some(t) if self.current_ns == Namespace::Html => get_text_parsing_mode_adjustment(t),
            _ if self.current_ns != Namespace::Html => {
                self.get_feedback_for_start_tag_in_foreign_content(tag_name_hash)
            }
            _ => TreeBuilderFeedback::None,
        }
    }

    pub fn get_feedback_for_end_tag_name(
        &mut self,
        tag_name_hash: Option<u64>,
    ) -> TreeBuilderFeedback {
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

    fn enter_ns(&mut self, ns: Namespace) -> TreeBuilderFeedback {
        self.ns_stack.push(ns);
        self.current_ns = ns;
        TreeBuilderFeedback::SetAllowCdata(ns != Namespace::Html)
    }

    fn leave_ns(&mut self) -> TreeBuilderFeedback {
        self.ns_stack.pop();

        self.current_ns = *self
            .ns_stack
            .last()
            .expect("Namespace stack should always have at least one item");

        TreeBuilderFeedback::SetAllowCdata(self.current_ns != Namespace::Html)
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
            None if prev_ns == Namespace::MathML =>TreeBuilderFeedback::RequestLexUnit(
                TreeBuilderSimulator::check_for_annotation_xml_end_tag_for_integration_point_exit_check,
            ),
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
            Some(t) if t == TagName::Font => TreeBuilderFeedback::RequestLexUnit(
                TreeBuilderSimulator::check_font_start_tag_token_for_foreign_content_exit,
            ),
            Some(t) if self.is_integration_point_enter(t) => TreeBuilderFeedback::RequestLexUnit(
                TreeBuilderSimulator::check_tag_self_closing_flag_for_integration_point_check
            ),
            // NOTE: integration point check <annotation-xml> case
            None if self.current_ns == Namespace::MathML => TreeBuilderFeedback::RequestLexUnit(
                TreeBuilderSimulator::check_for_annotation_xml_start_tag_for_integration_point_check,
            ),
            _ => TreeBuilderFeedback::None,
        }
    }

    fn check_tag_self_closing_flag_for_integration_point_check(
        &mut self,
        lex_unit: &LexUnit<'_>,
    ) -> TreeBuilderFeedback {
        match expect_token_view!(lex_unit) {
            TokenView::StartTag { self_closing, .. } => {
                if self_closing {
                    TreeBuilderFeedback::None
                } else {
                    self.enter_ns(Namespace::Html)
                }
            }
            _ => unreachable!("Token should be a start tag at this point"),
        }
    }

    fn check_for_annotation_xml_end_tag_for_integration_point_exit_check(
        &mut self,
        lex_unit: &LexUnit<'_>,
    ) -> TreeBuilderFeedback {
        match expect_token_view!(lex_unit) {
            TokenView::EndTag { name, .. } => {
                let name = lex_unit.input().slice(name);

                if eq_case_insensitive(&name, b"annotation-xml") {
                    self.leave_ns()
                } else {
                    TreeBuilderFeedback::None
                }
            }
            _ => unreachable!("Token should be an end tag at this point"),
        }
    }

    fn check_for_annotation_xml_start_tag_for_integration_point_check(
        &mut self,
        lex_unit: &LexUnit<'_>,
    ) -> TreeBuilderFeedback {
        match expect_token_view!(lex_unit) {
            TokenView::StartTag {
                name,
                ref attributes,
                self_closing,
                ..
            } => {
                let name = lex_unit.input().slice(name);

                if !self_closing && eq_case_insensitive(&name, b"annotation-xml") {
                    for attr in attributes.borrow().iter() {
                        let name = lex_unit.input().slice(attr.name);
                        let value = lex_unit.input().slice(attr.value);

                        if eq_case_insensitive(&name, b"encoding")
                            && (eq_case_insensitive(&value, b"text/html")
                                || eq_case_insensitive(&value, b"application/xhtml+xml"))
                        {
                            return self.enter_ns(Namespace::Html);
                        }
                    }
                }
            }
            _ => unreachable!("Token should be a start tag at this point"),
        }

        TreeBuilderFeedback::None
    }

    fn check_font_start_tag_token_for_foreign_content_exit(
        &mut self,
        lex_unit: &LexUnit<'_>,
    ) -> TreeBuilderFeedback {
        match expect_token_view!(lex_unit) {
            TokenView::StartTag { ref attributes, .. } => {
                for attr in attributes.borrow().iter() {
                    let name = lex_unit.input().slice(attr.name);

                    if eq_case_insensitive(&name, b"color")
                        || eq_case_insensitive(&name, b"size")
                        || eq_case_insensitive(&name, b"face")
                    {
                        return self.leave_ns();
                    }
                }
            }
            _ => unreachable!("Token should be a start tag at this point"),
        }

        TreeBuilderFeedback::None
    }
}

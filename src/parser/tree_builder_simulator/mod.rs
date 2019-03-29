// HTML parser has 6 different state machines for text parsing
// purposes in different contexts. Switch between these state machines
// usually performed by the tree construction stage depending on the
// state of the stack of open elements (HTML is a context-sensitive grammar).
//
// Luckily, in the majority of cases this tree construction stage feedback
// can be simulated without the stack of open elements and comlicated rules
// required to maintain its state.
//
// This module implements such feedback simulation. However, there are few
// cases where we can't unambiguously determine parsing context and prefer
// to bail out from the tokenization in such a case
// (see `AmbiguityGuard` for the details).
mod ambiguity_guard;

use self::ambiguity_guard::AmbiguityGuard;
use crate::base::Bytes;
use crate::parser::outputs::{TagLexeme, TagTokenOutline};
use crate::parser::{TagNameHash, TextType};
use TagTokenOutline::*;

pub use self::ambiguity_guard::AmbiguityGuardError;

const DEFAULT_NS_STACK_CAPACITY: usize = 256;

#[derive(Copy, Clone, Eq, PartialEq)]
enum Namespace {
    Html,
    Svg,
    MathML,
}

#[must_use]
pub enum TreeBuilderFeedback {
    SwitchTextType(TextType),
    SetAllowCdata(bool),
    RequestLexeme(Box<dyn FnMut(&mut TreeBuilderSimulator, &TagLexeme<'_>) -> TreeBuilderFeedback>),
    None,
}

impl From<TextType> for TreeBuilderFeedback {
    #[inline]
    fn from(text_type: TextType) -> Self {
        TreeBuilderFeedback::SwitchTextType(text_type)
    }
}

#[inline]
fn request_lexeme(
    callback: impl FnMut(&mut TreeBuilderSimulator, &TagLexeme<'_>) -> TreeBuilderFeedback + 'static,
) -> TreeBuilderFeedback {
    TreeBuilderFeedback::RequestLexeme(Box::new(callback))
}

macro_rules! expect_tag {
    ($lexeme:expr, $tag_pat:pat => $action:expr) => {
        match *$lexeme.token_outline() {
            $tag_pat => $action,
            _ => unreachable!("Got unexpected tag type"),
        }
    };
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
fn get_text_type_adjustment(tag_name_hash: u64) -> TreeBuilderFeedback {
    use TextType::*;

    if tag_is_one_of!(tag_name_hash, [Textarea, Title]) {
        RCData.into()
    } else if tag_name_hash == TagNameHash::Plaintext {
        PlainText.into()
    } else if tag_name_hash == TagNameHash::Script {
        ScriptData.into()
    } else if tag_is_one_of!(
        tag_name_hash,
        [Style, Iframe, Xmp, Noembed, Noframes, Noscript]
    ) {
        RawText.into()
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
    ambiguity_guard: AmbiguityGuard,
}

impl Default for TreeBuilderSimulator {
    fn default() -> Self {
        let mut simulator = TreeBuilderSimulator {
            ns_stack: Vec::with_capacity(DEFAULT_NS_STACK_CAPACITY),
            current_ns: Namespace::Html,
            ambiguity_guard: AmbiguityGuard::default(),
        };

        simulator.ns_stack.push(Namespace::Html);

        simulator
    }
}

impl TreeBuilderSimulator {
    pub fn get_feedback_for_start_tag(
        &mut self,
        tag_name_hash: Option<u64>,
        with_ambiguity_check: bool,
    ) -> Result<TreeBuilderFeedback, AmbiguityGuardError> {
        if with_ambiguity_check {
            self.ambiguity_guard.track_start_tag(tag_name_hash)?;
        }

        Ok(match tag_name_hash {
            Some(t) if t == TagNameHash::Svg => self.enter_ns(Namespace::Svg),
            Some(t) if t == TagNameHash::Math => self.enter_ns(Namespace::MathML),
            Some(t) if self.current_ns == Namespace::Html => get_text_type_adjustment(t),
            _ if self.current_ns != Namespace::Html => {
                self.get_feedback_for_start_tag_in_foreign_content(tag_name_hash)
            }
            _ => TreeBuilderFeedback::None,
        })
    }

    pub fn get_feedback_for_end_tag(
        &mut self,
        tag_name_hash: Option<u64>,
        with_ambiguity_check: bool,
    ) -> TreeBuilderFeedback {
        if with_ambiguity_check {
            self.ambiguity_guard.track_end_tag(tag_name_hash);
        }

        match tag_name_hash {
            Some(t) if self.current_ns == Namespace::Svg && t == TagNameHash::Svg => {
                self.leave_ns()
            }
            Some(t) if self.current_ns == Namespace::MathML && t == TagNameHash::Math => {
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
            None if prev_ns == Namespace::MathML => request_lexeme(|this, lexeme| {
                expect_tag!(lexeme, EndTag { name, .. } => {
                    if eq_case_insensitive(&lexeme.part(name), b"annotation-xml") {
                        this.leave_ns()
                    } else {
                        TreeBuilderFeedback::None
                    }
                })
            }),

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
            Some(t) if t == TagNameHash::Font => request_lexeme(|this, lexeme| {
                expect_tag!(lexeme, StartTag { ref attributes, .. } => {
                    for attr in attributes.borrow().iter() {
                        let name = lexeme.part(attr.name);

                        if eq_case_insensitive(&name, b"color")
                            || eq_case_insensitive(&name, b"size")
                            || eq_case_insensitive(&name, b"face")
                        {
                            return this.leave_ns();
                        }
                    }
                });

                TreeBuilderFeedback::None
            }),

            Some(t) if self.is_integration_point_enter(t) => request_lexeme(|this, lexeme| {
                expect_tag!(lexeme, StartTag { self_closing, .. } => {
                    if self_closing {
                        TreeBuilderFeedback::None
                    } else {
                        this.enter_ns(Namespace::Html)
                    }
                })
            }),

            // NOTE: integration point check <annotation-xml> case
            None if self.current_ns == Namespace::MathML => request_lexeme(|this, lexeme| {
                expect_tag!(lexeme, StartTag {
                    name,
                    ref attributes,
                    self_closing,
                    ..
                } => {
                    let name = lexeme.part(name);

                    if !self_closing && eq_case_insensitive(&name, b"annotation-xml") {
                        for attr in attributes.borrow().iter() {
                            let name = lexeme.part(attr.name);
                            let value = lexeme.part(attr.value);

                            if eq_case_insensitive(&name, b"encoding")
                                && (eq_case_insensitive(&value, b"text/html")
                                    || eq_case_insensitive(&value, b"application/xhtml+xml"))
                            {
                                return this.enter_ns(Namespace::Html);
                            }
                        }
                    }
                });

                TreeBuilderFeedback::None
            }),

            _ => TreeBuilderFeedback::None,
        }
    }
}

//! HTML parser has 6 different state machines for text parsing
//! purposes in different contexts. Switch between these state machines
//! usually performed by the tree construction stage depending on the
//! state of the stack of open elements (HTML is a context-sensitive grammar).
//!
//! Luckily, in the majority of cases this tree construction stage feedback
//! can be simulated without the stack of open elements and complicated rules
//! required to maintain its state.
//!
//! This module implements such feedback simulation. However, there are few
//! cases where we can't unambiguously determine parsing context and prefer
//! to bail out from the tokenization in such a case
//! (see `AmbiguityGuard` for the details).
mod ambiguity_guard;
mod integration_points;

use self::ambiguity_guard::AmbiguityGuard;
use self::integration_points::IntegrationPointStack;
use crate::base::eq_case_insensitive;
use crate::html::{LocalNameHash, Namespace, Tag, TextType};
use crate::memory::{MemoryLimitExceededError, SharedMemoryLimiter};
use crate::parser::{TagLexeme, TagTokenOutline};
use TagTokenOutline::StartTag;

pub use self::ambiguity_guard::ParsingAmbiguityError;

const DEFAULT_NS_STACK_CAPACITY: usize = 256;

#[must_use]
pub(crate) enum TreeBuilderFeedback {
    SwitchTextType(TextType),
    SetAllowCdata(bool),
    #[allow(clippy::type_complexity)]
    RequestLexeme(Box<dyn FnMut(&mut TreeBuilderSimulator, &TagLexeme<'_>) -> Self + Send>),
    MemoryLimitExceeded(MemoryLimitExceededError),
    None,
}

impl From<TextType> for TreeBuilderFeedback {
    #[inline]
    fn from(text_type: TextType) -> Self {
        Self::SwitchTextType(text_type)
    }
}

#[inline]
fn request_lexeme(
    callback: impl FnMut(&mut TreeBuilderSimulator, &TagLexeme<'_>) -> TreeBuilderFeedback
    + 'static
    + Send,
) -> TreeBuilderFeedback {
    TreeBuilderFeedback::RequestLexeme(Box::new(callback))
}

macro_rules! expect_tag {
    ($lexeme:expr, $tag_pat:pat => $action:expr) => {
        match *$lexeme.token_outline() {
            $tag_pat => $action,
            _ => {
                debug_assert!(false, "Got unexpected tag type");
                return TreeBuilderFeedback::None;
            }
        }
    };
}

#[inline]
fn get_text_type_adjustment(tag_name: LocalNameHash) -> TreeBuilderFeedback {
    use TextType::*;

    if tag_is_one_of!(tag_name, [Textarea, Title]) {
        RCData.into()
    } else if tag_name == Tag::Plaintext {
        PlainText.into()
    } else if tag_name == Tag::Script {
        ScriptData.into()
    } else if tag_is_one_of!(tag_name, [Style, Iframe, Xmp, Noembed, Noframes, Noscript]) {
        RawText.into()
    } else {
        TreeBuilderFeedback::None
    }
}

#[inline]
fn causes_foreign_content_exit(tag_name: LocalNameHash) -> bool {
    tag_is_one_of!(
        tag_name,
        [
            B, Big, Blockquote, Body, Br, Center, Code, Dd, Div, Dl, Dt, Em, Embed, H1, H2, H3, H4,
            H5, H6, Head, Hr, I, Img, Li, Listing, Menu, Meta, Nobr, Ol, P, Pre, Ruby, S, Small,
            Span, Strong, Strike, Sub, Sup, Table, Tt, U, Ul, Var
        ]
    )
}

#[inline]
fn is_text_integration_point_in_math_ml(tag_name: LocalNameHash) -> bool {
    tag_is_one_of!(tag_name, [Mi, Mo, Mn, Ms, Mtext])
}

#[inline]
fn is_html_integration_point_in_svg(tag_name: LocalNameHash) -> bool {
    tag_is_one_of!(tag_name, [Desc, Title, ForeignObject])
}

#[inline]
fn is_html_ancestor(tag_name: LocalNameHash) -> bool {
    !tag_is_one_of!(
        tag_name,
        [
            Area, Base, Basefont, Bgsound, Br, Col, Embed, Hr, Img, Input, Keygen, Link, Meta,
            Param, Source, Track, Wbr, Head, Body
        ]
    ) && tag_name != LocalNameHash::from("html")
}

// TODO limit ns stack
pub(crate) struct TreeBuilderSimulator {
    ns_stack: Vec<Namespace>,
    current_ns: Namespace,
    element_namespace_override: Option<Namespace>,
    integration_points: IntegrationPointStack,
    ambiguity_guard: AmbiguityGuard,
    strict: bool,
}

impl TreeBuilderSimulator {
    #[inline]
    #[must_use]
    pub fn new(strict: bool, memory_limiter: SharedMemoryLimiter) -> Self {
        let mut simulator = Self {
            ns_stack: Vec::with_capacity(DEFAULT_NS_STACK_CAPACITY),
            current_ns: Namespace::Html,
            element_namespace_override: None,
            integration_points: IntegrationPointStack::new(memory_limiter),
            ambiguity_guard: AmbiguityGuard::default(),
            strict,
        };

        simulator.ns_stack.push(Namespace::Html);

        simulator
    }

    pub fn get_feedback_for_start_tag(
        &mut self,
        tag_name: LocalNameHash,
        name: &[u8],
    ) -> Result<TreeBuilderFeedback, ParsingAmbiguityError> {
        if self.strict {
            self.ambiguity_guard.track_start_tag(tag_name)?;
        }

        Ok(if tag_name == Tag::Svg {
            self.enter_foreign_root(Namespace::Svg)
        } else if tag_name == Tag::Math {
            self.enter_foreign_root(Namespace::MathML)
        } else if self.current_ns != Namespace::Html {
            self.get_feedback_for_start_tag_in_foreign_content(tag_name)
        } else {
            self.get_feedback_for_start_tag_in_html(tag_name, name)
        })
    }

    pub fn get_feedback_for_end_tag(
        &mut self,
        tag_name: LocalNameHash,
        name: &[u8],
    ) -> TreeBuilderFeedback {
        if self.strict {
            self.ambiguity_guard.track_end_tag(tag_name);
        }

        if self.current_ns == Namespace::Html {
            self.check_integration_point_exit(tag_name, name)
        } else if self.should_leave_ns(tag_name) {
            match tag_is_one_of!(tag_name, [P, Br]) {
                true => {
                    let feedback = self.leave_foreign_content();
                    let _ = self.check_integration_point_exit(tag_name, name);

                    feedback
                }
                false => self.leave_ns(),
            }
        } else {
            TreeBuilderFeedback::None
        }
    }

    fn should_leave_ns(&self, tag_name: LocalNameHash) -> bool {
        if self.current_ns == Namespace::Svg && tag_name == Tag::Svg
            || self.current_ns == Namespace::MathML && tag_name == Tag::Math
        {
            return true;
        }

        if (self.current_ns == Namespace::Svg || self.current_ns == Namespace::MathML)
            && tag_is_one_of!(tag_name, [P, Br])
        {
            // 13.2.6.5
            return true;
        }
        false
    }

    #[inline]
    pub const fn current_ns(&self) -> Namespace {
        self.current_ns
    }

    #[inline]
    pub fn take_element_namespace(&mut self) -> Namespace {
        self.element_namespace_override
            .take()
            .unwrap_or(self.current_ns)
    }

    fn enter_foreign_root(&mut self, namespace: Namespace) -> TreeBuilderFeedback {
        request_lexeme(move |this, lexeme| {
            expect_tag!(lexeme, StartTag { self_closing, .. } => match self_closing {
                true => {
                    // The element is foreign even though it does not change the following context.
                    this.element_namespace_override = Some(namespace);
                    TreeBuilderFeedback::None
                }
                false => this.enter_ns(namespace),
            })
        })
    }

    #[inline]
    fn enter_ns(&mut self, ns: Namespace) -> TreeBuilderFeedback {
        self.ns_stack.push(ns);
        self.current_ns = ns;
        TreeBuilderFeedback::SetAllowCdata(ns != Namespace::Html)
    }

    #[inline]
    fn leave_ns(&mut self) -> TreeBuilderFeedback {
        self.ns_stack.pop();

        let Some(item) = self.ns_stack.last() else {
            debug_assert!(
                false,
                "Namespace stack should always have at least one item"
            );
            return TreeBuilderFeedback::None;
        };
        self.current_ns = *item;

        TreeBuilderFeedback::SetAllowCdata(self.current_ns != Namespace::Html)
    }

    #[inline]
    fn leave_foreign_content(&mut self) -> TreeBuilderFeedback {
        while self.current_ns != Namespace::Html {
            let _ = self.leave_ns();
        }

        TreeBuilderFeedback::SetAllowCdata(false)
    }

    fn leave_foreign_content_for_start_tag(
        &mut self,
        tag_name: LocalNameHash,
    ) -> TreeBuilderFeedback {
        let feedback = self.leave_foreign_content();

        if self.has_current_integration_point() && is_html_ancestor(tag_name) {
            let descendant_feedback = self.track_integration_point_descendant(tag_name, b"");

            if matches!(
                descendant_feedback,
                TreeBuilderFeedback::MemoryLimitExceeded(_)
            ) {
                return descendant_feedback;
            }
        }

        feedback
    }

    fn is_integration_point_enter(&self, tag_name: LocalNameHash) -> bool {
        self.current_ns == Namespace::Svg && is_html_integration_point_in_svg(tag_name)
            || self.current_ns == Namespace::MathML
                && is_text_integration_point_in_math_ml(tag_name)
    }

    fn enter_integration_point(&mut self, tag_name: LocalNameHash) -> TreeBuilderFeedback {
        let name = match tag_name.is_empty() {
            true => b"annotation-xml".as_slice(),
            false => b"".as_slice(),
        };

        if let Err(error) = self
            .integration_points
            .push(tag_name, name, self.ns_stack.len() + 1)
        {
            return TreeBuilderFeedback::MemoryLimitExceeded(error);
        }

        self.element_namespace_override = Some(self.current_ns);
        self.enter_ns(Namespace::Html)
    }

    #[inline]
    fn has_current_integration_point(&self) -> bool {
        self.integration_points
            .last()
            .is_some_and(|entry| entry.namespace_depth == self.ns_stack.len())
    }

    fn get_feedback_for_start_tag_in_html(
        &mut self,
        tag_name: LocalNameHash,
        name: &[u8],
    ) -> TreeBuilderFeedback {
        if !self.has_current_integration_point() {
            return get_text_type_adjustment(tag_name);
        }

        if !is_html_ancestor(tag_name) {
            return get_text_type_adjustment(tag_name);
        }

        self.track_integration_point_descendant(tag_name, name)
    }

    fn track_integration_point_descendant(
        &mut self,
        tag_name: LocalNameHash,
        name: &[u8],
    ) -> TreeBuilderFeedback {
        match self
            .integration_points
            .push(tag_name, name, self.ns_stack.len())
        {
            Ok(()) => get_text_type_adjustment(tag_name),
            Err(error) => TreeBuilderFeedback::MemoryLimitExceeded(error),
        }
    }

    fn check_integration_point_exit(
        &mut self,
        tag_name: LocalNameHash,
        name: &[u8],
    ) -> TreeBuilderFeedback {
        if !self.has_current_integration_point() {
            return TreeBuilderFeedback::None;
        }

        if self
            .integration_points
            .close(tag_name, name, self.ns_stack.len())
        {
            self.leave_ns()
        } else {
            TreeBuilderFeedback::None
        }
    }

    fn get_feedback_for_start_tag_in_foreign_content(
        &mut self,
        tag_name: LocalNameHash,
    ) -> TreeBuilderFeedback {
        if causes_foreign_content_exit(tag_name) {
            return self.leave_foreign_content_for_start_tag(tag_name);
        }

        if self.is_integration_point_enter(tag_name) {
            return request_lexeme(move |this, lexeme| {
                expect_tag!(lexeme, StartTag { self_closing, .. } => {
                    if self_closing {
                        TreeBuilderFeedback::None
                    } else {
                        this.enter_integration_point(tag_name)
                    }
                })
            });
        }

        if tag_name == Tag::Font {
            // NOTE: <font> tag special case requires attributes
            // to decide on foreign context exit
            return request_lexeme(move |this, lexeme| {
                expect_tag!(lexeme, StartTag { ref attributes, .. } => {
                    for attr in attributes.iter() {
                        let name = lexeme.part(attr.name);

                        if eq_case_insensitive(&name, b"color")
                            || eq_case_insensitive(&name, b"size")
                            || eq_case_insensitive(&name, b"face")
                        {
                            return this.leave_foreign_content_for_start_tag(tag_name);
                        }
                    }
                });

                TreeBuilderFeedback::None
            });
        }

        if tag_name.is_empty() && self.current_ns == Namespace::MathML {
            // NOTE: tag name hash is empty - we need integration point check
            // for the possible <annotation-xml> case
            return request_lexeme(move |this, lexeme| {
                expect_tag!(lexeme, StartTag {
                    name,
                    ref attributes,
                    self_closing,
                    ..
                } => {
                    let name = lexeme.part(name);

                    if !self_closing && eq_case_insensitive(&name, b"annotation-xml") {
                        for attr in attributes.iter() {
                            let name = lexeme.part(attr.name);
                            let Some(value) = lexeme.opt_part(attr.value) else {
                                continue;
                            };

                            if eq_case_insensitive(&name, b"encoding")
                                && (eq_case_insensitive(&value, b"text/html")
                                    || eq_case_insensitive(&value, b"application/xhtml+xml"))
                            {
                                return this.enter_integration_point(tag_name);
                            }
                        }
                    }
                });

                TreeBuilderFeedback::None
            });
        }

        TreeBuilderFeedback::None
    }
}

#[cfg(test)]
mod tests {
    use super::integration_points::IntegrationPoint;
    use super::*;

    fn integration_metadata_limit() -> usize {
        let limiter = SharedMemoryLimiter::new(usize::MAX);
        let mut simulator = TreeBuilderSimulator::new(true, limiter.clone());
        let _ = simulator.enter_ns(Namespace::Svg);
        let _ = simulator.enter_integration_point("desc".into());

        limiter.current_usage()
    }

    /// Plain HTML must not allocate the integration-point lifetime stack.
    #[test]
    fn test_tree_builder_simulator_namespace_advisories_plain_html_metadata() {
        let limiter = SharedMemoryLimiter::new(0);
        let mut simulator = TreeBuilderSimulator::new(true, limiter.clone());

        for _ in 0..1024 {
            let _ = simulator
                .get_feedback_for_start_tag("div".into(), b"div")
                .unwrap();
            let _ = simulator.get_feedback_for_end_tag("div".into(), b"div");
        }

        assert_eq!(
            simulator.integration_points.len(),
            0,
            "HTML needs no lifetime entries"
        );
        assert_eq!(
            limiter.current_usage(),
            0,
            "HTML must not allocate lifetime metadata"
        );
    }

    /// Closing points retains charged reusable capacity, and dropping the parser releases it.
    #[test]
    fn test_tree_builder_simulator_namespace_advisories_integration_metadata_released() {
        let limiter = SharedMemoryLimiter::new(4096);
        let mut simulator = TreeBuilderSimulator::new(true, limiter.clone());
        let _ = simulator.enter_ns(Namespace::Svg);
        let _ = simulator.enter_integration_point("desc".into());

        assert_eq!(
            simulator.take_element_namespace(),
            Namespace::Svg,
            "the integration opener must be reported as foreign"
        );
        for _ in 0..33 {
            let _ = simulator
                .get_feedback_for_start_tag("desc".into(), b"desc")
                .unwrap();
        }

        let charged = limiter.current_usage();

        assert!(
            charged >= 34 * size_of::<IntegrationPoint>(),
            "every retained integration-point occurrence must be charged"
        );
        for _ in 0..33 {
            let _ = simulator.get_feedback_for_end_tag("desc".into(), b"desc");

            assert_eq!(
                simulator.current_ns(),
                Namespace::Html,
                "same-named descendants must not close the integration frame"
            );
        }

        let _ = simulator.get_feedback_for_end_tag("desc".into(), b"desc");

        assert_eq!(
            simulator.current_ns(),
            Namespace::Svg,
            "the real closer restores foreign content"
        );
        assert_eq!(
            simulator.integration_points.len(),
            0,
            "all occurrences must be removed"
        );
        assert_eq!(
            limiter.current_usage(),
            charged,
            "retained capacity must remain charged"
        );
        drop(simulator);
        assert_eq!(
            limiter.current_usage(),
            0,
            "drop must release the entire metadata charge"
        );
    }

    /// Failing to reserve the opener must leave the namespace stack and override unchanged.
    #[test]
    fn test_tree_builder_simulator_namespace_advisories_integration_opener_allocation_failure() {
        let mut simulator = TreeBuilderSimulator::new(true, SharedMemoryLimiter::new(0));
        let _ = simulator.enter_ns(Namespace::Svg);
        let feedback = simulator.enter_integration_point("desc".into());

        assert!(
            matches!(feedback, TreeBuilderFeedback::MemoryLimitExceeded(_)),
            "an exhausted limit must reject the integration opener"
        );
        assert_eq!(
            simulator.ns_stack.len(),
            2,
            "reservation must precede the namespace push"
        );
        assert_eq!(
            simulator.current_ns(),
            Namespace::Svg,
            "failed entry must keep foreign context"
        );
        assert_eq!(
            simulator.element_namespace_override, None,
            "failed entry must not leave an override"
        );
        assert_eq!(
            simulator.integration_points.len(),
            0,
            "failed entry must not retain metadata"
        );
    }

    /// Same-named descendants must not grow beyond the charged metadata capacity.
    #[test]
    fn test_tree_builder_simulator_namespace_advisories_integration_descendant_allocation_failure()
    {
        let limit = integration_metadata_limit();
        let mut simulator = TreeBuilderSimulator::new(true, SharedMemoryLimiter::new(limit));
        let _ = simulator.enter_ns(Namespace::Svg);
        let _ = simulator.enter_integration_point("desc".into());
        let _ = simulator.take_element_namespace();

        for _ in 0..7 {
            let feedback = simulator
                .get_feedback_for_start_tag("desc".into(), b"desc")
                .unwrap();

            assert!(
                matches!(feedback, TreeBuilderFeedback::None),
                "reserved entries must succeed"
            );
        }

        let feedback = simulator
            .get_feedback_for_start_tag("desc".into(), b"desc")
            .unwrap();

        assert!(
            matches!(feedback, TreeBuilderFeedback::MemoryLimitExceeded(_)),
            "growing past the shared memory limit must fail"
        );
        assert_eq!(
            simulator.integration_points.len(),
            8,
            "failed growth must preserve existing entries"
        );
        assert_eq!(
            simulator.ns_stack.len(),
            3,
            "descendant accounting must not change namespace depth"
        );
        assert_eq!(
            simulator.current_ns(),
            Namespace::Html,
            "failed growth must preserve the context"
        );
    }

    /// Sequential integration points must reuse capacity rather than accumulate lifetime records.
    #[test]
    fn test_tree_builder_simulator_namespace_advisories_integration_metadata_reused() {
        let limiter = SharedMemoryLimiter::new(integration_metadata_limit());
        let mut simulator = TreeBuilderSimulator::new(true, limiter.clone());
        let _ = simulator.enter_ns(Namespace::Svg);

        for _ in 0..1024 {
            let feedback = simulator.enter_integration_point("desc".into());

            assert!(
                matches!(feedback, TreeBuilderFeedback::SetAllowCdata(false)),
                "sequential points must fit within one metadata allocation"
            );
            let _ = simulator.take_element_namespace();
            let _ = simulator.get_feedback_for_end_tag("desc".into(), b"desc");

            assert_eq!(
                simulator.integration_points.len(),
                0,
                "closed points must remove their records"
            );
        }

        drop(simulator);
        assert_eq!(
            limiter.current_usage(),
            0,
            "successful reuse must not leak memory charges"
        );
    }
}

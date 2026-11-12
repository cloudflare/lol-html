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

const DEFAULT_FRAME_STACK_CAPACITY: usize = 256;

#[must_use]
pub(crate) enum TreeBuilderFeedback {
    SwitchTextType(TextType),
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

// TODO limit frame stack
pub(crate) struct TreeBuilderSimulator {
    frames: Vec<Frame>,
    current_ns: Namespace,
    element_namespace_override: Option<Namespace>,
    integration_points: IntegrationPointStack,
    ambiguity_guard: AmbiguityGuard,
    strict: bool,
}

/// A parsing-context frame: the element that opened it.
struct Frame {
    /// Namespace the frame's content is parsed in.
    namespace: Namespace,
    kind: FrameKind,
}

enum FrameKind {
    /// The document frame, never popped.
    Root,
    /// A foreign element whose end tag has to pop this frame (`svg`, `math`).
    Foreign(LocalNameHash),
    /// A foreign element whose children are parsed as HTML.
    IntegrationPoint,
}

impl TreeBuilderSimulator {
    #[inline]
    #[must_use]
    pub fn new(strict: bool, memory_limiter: SharedMemoryLimiter) -> Self {
        let mut simulator = Self {
            frames: Vec::with_capacity(DEFAULT_FRAME_STACK_CAPACITY),
            current_ns: Namespace::Html,
            element_namespace_override: None,
            integration_points: IntegrationPointStack::new(memory_limiter),
            ambiguity_guard: AmbiguityGuard::default(),
            strict,
        };

        simulator.frames.push(Frame {
            namespace: Namespace::Html,
            kind: FrameKind::Root,
        });

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
            self.enter_foreign_root(Namespace::Svg, tag_name)
        } else if tag_name == Tag::Math {
            self.enter_foreign_root(Namespace::MathML, tag_name)
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
        } else if tag_is_one_of!(tag_name, [P, Br]) {
            // 13.2.6.5: `</p>` and `</br>` pop every foreign element up to the
            // nearest HTML element or integration point.
            let feedback = self.leave_foreign_content();
            let _ = self.check_integration_point_exit(tag_name, name);

            feedback
        } else {
            self.leave_foreign_frame(tag_name);

            TreeBuilderFeedback::None
        }
    }

    /// 13.2.6.5 "Any other end tag": pops the matching foreign element and
    /// every foreign element opened after it. The walk stops at the nearest
    /// integration point or the document frame, so an end tag cannot leave the
    /// current foreign context through an HTML boundary.
    fn leave_foreign_frame(&mut self, tag_name: LocalNameHash) {
        for pos in (1..self.frames.len()).rev() {
            match &self.frames[pos].kind {
                FrameKind::Foreign(open) if *open == tag_name => {
                    self.frames.truncate(pos);
                    self.current_ns = self.frames[pos - 1].namespace;

                    return;
                }
                FrameKind::Foreign(_) => (),
                _ => return,
            }
        }
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

    /// CDATA sections are only allowed when the adjusted current node is not an
    /// HTML element (13.2.5.1 "markup declaration open state").
    ///
    /// The element that opened an integration point stays foreign, so a CDATA
    /// section directly in it is text. An HTML element opened inside it becomes
    /// the current node, and there `<![CDATA[` is a bogus comment.
    #[inline]
    pub fn allow_cdata(&self) -> bool {
        if self.current_ns != Namespace::Html {
            return true;
        }

        self.has_current_integration_point()
            && self.integration_points.current_is_opener(self.frames.len())
    }

    fn enter_foreign_root(
        &mut self,
        namespace: Namespace,
        tag_name: LocalNameHash,
    ) -> TreeBuilderFeedback {
        request_lexeme(move |this, lexeme| {
            expect_tag!(lexeme, StartTag { self_closing, .. } => match self_closing {
                true => {
                    // The element is foreign even though it does not change the following context.
                    this.element_namespace_override = Some(namespace);
                    TreeBuilderFeedback::None
                }
                false => this.enter_ns(namespace, FrameKind::Foreign(tag_name)),
            })
        })
    }

    #[inline]
    fn enter_ns(&mut self, ns: Namespace, kind: FrameKind) -> TreeBuilderFeedback {
        self.frames.push(Frame {
            namespace: ns,
            kind,
        });
        self.current_ns = ns;
        TreeBuilderFeedback::None
    }

    #[inline]
    fn leave_ns(&mut self) -> TreeBuilderFeedback {
        self.frames.pop();

        let Some(frame) = self.frames.last() else {
            debug_assert!(false, "Frame stack should always have at least one item");
            return TreeBuilderFeedback::None;
        };
        self.current_ns = frame.namespace;

        TreeBuilderFeedback::None
    }

    #[inline]
    fn leave_foreign_content(&mut self) -> TreeBuilderFeedback {
        while self.current_ns != Namespace::Html {
            let _ = self.leave_ns();
        }

        TreeBuilderFeedback::None
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
            .push(tag_name, name, self.frames.len() + 1)
        {
            return TreeBuilderFeedback::MemoryLimitExceeded(error);
        }

        self.element_namespace_override = Some(self.current_ns);
        self.enter_ns(Namespace::Html, FrameKind::IntegrationPoint)
    }

    #[inline]
    fn has_current_integration_point(&self) -> bool {
        matches!(
            self.frames.last().map(|frame| &frame.kind),
            Some(FrameKind::IntegrationPoint)
        )
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
            .push(tag_name, name, self.frames.len())
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
            .close(tag_name, name, self.frames.len())
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
        let _ = simulator.enter_ns(Namespace::Svg, FrameKind::Foreign("svg".into()));
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
        let _ = simulator.enter_ns(Namespace::Svg, FrameKind::Foreign("svg".into()));
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
        let _ = simulator.enter_ns(Namespace::Svg, FrameKind::Foreign("svg".into()));
        let feedback = simulator.enter_integration_point("desc".into());

        assert!(
            matches!(feedback, TreeBuilderFeedback::MemoryLimitExceeded(_)),
            "an exhausted limit must reject the integration opener"
        );
        assert_eq!(
            simulator.frames.len(),
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
        let _ = simulator.enter_ns(Namespace::Svg, FrameKind::Foreign("svg".into()));
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
            simulator.frames.len(),
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
        let _ = simulator.enter_ns(Namespace::Svg, FrameKind::Foreign("svg".into()));

        for _ in 0..1024 {
            let feedback = simulator.enter_integration_point("desc".into());

            assert!(
                matches!(feedback, TreeBuilderFeedback::None),
                "sequential points must fit within one metadata allocation"
            );
            assert!(
                simulator.allow_cdata(),
                "a CDATA section directly in the opener is text"
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

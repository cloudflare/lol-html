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
//!
//! The simulator tracks two different things, which are easy to confuse:
//!
//! * `ParseMode` — the insertion mode that decides how the *content* of the
//!   current node is tokenized. HTML content is tokenized with the HTML rules
//!   (raw text switching, `<!--` comments, `/>` ignored), foreign content with
//!   the foreign rules (markup inside `<svg><script>`, CDATA sections, `/>`
//!   honoured).
//! * `Namespace` — the DOM namespace of an individual element. It is the
//!   insertion mode for everything except HTML integration points, whose
//!   element is foreign while their children are HTML.
//!
//! They differ exactly at integration points: `<svg><desc>` opens an SVG
//! `<desc>` element (namespace `Svg`) whose children are parsed as HTML.
mod ambiguity_guard;

use self::ambiguity_guard::AmbiguityGuard;
use crate::base::eq_case_insensitive;
use crate::html::{LocalNameHash, Namespace, Tag, TextType};
use crate::parser::{TagLexeme, TagTokenOutline};
use TagTokenOutline::{EndTag, StartTag};

pub use self::ambiguity_guard::ParsingAmbiguityError;

const DEFAULT_FRAME_STACK_CAPACITY: usize = 64;

/// Insertion mode for the content that follows the current node.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum ParseMode {
    Html,
    Svg,
    MathML,
}

impl ParseMode {
    #[inline]
    const fn namespace(self) -> Namespace {
        match self {
            Self::Html => Namespace::Html,
            Self::Svg => Namespace::Svg,
            Self::MathML => Namespace::MathML,
        }
    }
}

/// An "integration point" (13.2.6.5): a foreign element whose children are
/// parsed with the HTML rules. The element itself stays foreign.
#[derive(Copy, Clone, Debug)]
enum IntegrationPoint {
    Svg(LocalNameHash),
    MathMlText(LocalNameHash),
    AnnotationXml,
}

impl IntegrationPoint {
    #[inline]
    fn element_name(self) -> Option<LocalNameHash> {
        match self {
            Self::Svg(name) | Self::MathMlText(name) => Some(name),
            Self::AnnotationXml => None,
        }
    }
}

struct IpState {
    kind: IntegrationPoint,
    /// HTML elements opened inside the integration point, innermost last.
    ///
    /// They are needed because the end tag of an HTML element nested in an
    /// integration point must not close the integration point (13.2.6.4.7
    /// "any other end tag"), and because CDATA permission depends on whether
    /// the current node is one of them.
    html_elements: Vec<LocalNameHash>,
}

enum FrameKind {
    Root,
    /// A foreign element whose end tag has to pop this frame (`svg`, `math`).
    Foreign(LocalNameHash),
    IntegrationPoint(Box<IpState>),
}

struct Frame {
    mode: ParseMode,
    /// Namespace of the element that opened this frame.
    element_ns: Namespace,
    kind: FrameKind,
}

impl Frame {
    #[inline]
    fn integration_point(&self) -> Option<&IpState> {
        match &self.kind {
            FrameKind::IntegrationPoint(state) => Some(state),
            _ => None,
        }
    }
}

#[must_use]
pub(crate) enum TreeBuilderFeedback {
    SwitchTextType(TextType),
    #[allow(clippy::type_complexity)]
    RequestLexeme(Box<dyn FnMut(&mut TreeBuilderSimulator, &TagLexeme<'_>) -> Self + Send>),
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

/// 13.2.6.5 "in foreign content": start tags that break out of foreign content.
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

/// 13.1.2.1 "Void elements": elements that never stay on the stack of open
/// elements, so they never become the current node.
#[inline]
fn is_void_tag(tag_name: LocalNameHash) -> bool {
    tag_is_one_of!(
        tag_name,
        [
            Area, Base, Br, Col, Embed, Hr, Img, Input, Link, Meta, Param, Source, Track, Wbr
        ]
    )
}

/// 13.2.6.4.7 "in body": elements in the special category. The "any other end
/// tag" rule stops (and ignores the token) at the first one of these.
#[inline]
fn is_special_html_element(tag_name: LocalNameHash) -> bool {
    tag_is_one_of!(
        tag_name,
        [
            Address, Applet, Area, Article, Aside, Base, Basefont, Bgsound, Blockquote, Body, Br,
            Button, Caption, Center, Col, Colgroup, Dd, Details, Dir, Div, Dl, Dt, Embed,
            Fieldset, Figcaption, Figure, Footer, Form, Frame, Frameset, H1, H2, H3, H4, H5, H6,
            Head, Header, Hgroup, Hr, Html, Iframe, Img, Input, Keygen, Li, Link, Listing, Main,
            Marquee, Menu, Meta, Nav, Noembed, Noframes, Noscript, Object, Ol, P, Param, Plaintext,
            Pre, Script, Search, Section, Select, Source, Style, Summary, Table, Tbody, Td,
            Template, Textarea, Tfoot, Th, Thead, Title, Tr, Track, Ul, Wbr, Xmp
        ]
    )
}

/// End tags that have their own rule in 13.2.6.4.7 "in body" and pop elements
/// until the matching one is found, including elements in the special category
/// (they generate implied end tags on the way).
#[inline]
fn pops_through_specials(tag_name: LocalNameHash) -> bool {
    tag_is_one_of!(
        tag_name,
        [
            Address, Applet, Article, Aside, Blockquote, Button, Center, Details, Dialog, Dir, Div,
            Dl, Dd, Dt, Fieldset, Figcaption, Figure, Footer, Form, H1, H2, H3, H4, H5, H6, Header,
            Hgroup, Li, Listing, Main, Marquee, Menu, Nav, Object, Ol, P, Pre, Section, Summary, Ul
        ]
    )
}

// TODO limit frame stack
pub(crate) struct TreeBuilderSimulator {
    frames: Vec<Frame>,
    current_element_ns: Namespace,
    ambiguity_guard: AmbiguityGuard,
    strict: bool,
}

impl TreeBuilderSimulator {
    #[inline]
    #[must_use]
    pub fn new(strict: bool) -> Self {
        let mut frames = Vec::with_capacity(DEFAULT_FRAME_STACK_CAPACITY);

        // NOTE: the root frame is never popped
        frames.push(Frame {
            mode: ParseMode::Html,
            element_ns: Namespace::Html,
            kind: FrameKind::Root,
        });

        Self {
            frames,
            current_element_ns: Namespace::Html,
            ambiguity_guard: AmbiguityGuard::default(),
            strict,
        }
    }

    #[inline]
    fn mode(&self) -> ParseMode {
        // NOTE: the frame stack always has the root frame
        self.frames[self.frames.len() - 1].mode
    }

    /// Namespace of the element created by the last start tag.
    #[inline]
    pub const fn current_ns(&self) -> Namespace {
        self.current_element_ns
    }

    /// CDATA sections are only allowed when the adjusted current node is not an
    /// HTML element (13.2.5.1 "markup declaration open state").
    #[inline]
    pub fn allow_cdata(&self) -> bool {
        let frame = &self.frames[self.frames.len() - 1];

        match frame.integration_point() {
            // NOTE: HTML elements opened inside the integration point are the
            // current node, and they are in the HTML namespace.
            Some(state) if !state.html_elements.is_empty() => false,
            _ => frame.element_ns != Namespace::Html,
        }
    }

    pub fn get_feedback_for_start_tag(
        &mut self,
        tag_name: LocalNameHash,
    ) -> Result<TreeBuilderFeedback, ParsingAmbiguityError> {
        if self.strict {
            self.ambiguity_guard.track_start_tag(tag_name)?;
        }

        Ok(if self.mode() == ParseMode::Html {
            self.start_tag_in_html_content(tag_name)
        } else {
            self.start_tag_in_foreign_content(tag_name)
        })
    }

    fn start_tag_in_html_content(&mut self, tag_name: LocalNameHash) -> TreeBuilderFeedback {
        // 13.2.6.4.7 "in body": `<svg>`/`<math>` insert a foreign element in the
        // SVG/MathML namespace. The self-closing flag pops it immediately.
        let ns = if tag_name == Tag::Svg {
            Namespace::Svg
        } else if tag_name == Tag::Math {
            Namespace::MathML
        } else {
            self.current_element_ns = Namespace::Html;
            self.note_html_element(tag_name);

            return get_text_type_adjustment(tag_name);
        };

        self.current_element_ns = ns;

        let mode = if ns == Namespace::Svg {
            ParseMode::Svg
        } else {
            ParseMode::MathML
        };

        request_lexeme(move |this, lexeme| {
            expect_tag!(lexeme, StartTag { self_closing, .. } => {
                if !self_closing {
                    this.push_frame(mode, ns, FrameKind::Foreign(tag_name));
                }
                TreeBuilderFeedback::None
            })
        })
    }

    fn start_tag_in_foreign_content(&mut self, tag_name: LocalNameHash) -> TreeBuilderFeedback {
        let foreign_ns = self.mode().namespace();

        // 13.2.6.5: a breakout start tag pops every foreign element up to the
        // nearest HTML element or integration point, then reprocesses in HTML.
        if causes_foreign_content_exit(tag_name) {
            self.pop_to_html_content();
            self.current_element_ns = Namespace::Html;
            self.note_html_element(tag_name);

            return TreeBuilderFeedback::None;
        }

        // 13.2.6.5: the integration point element itself stays foreign, but its
        // children are parsed with the HTML rules.
        if let Some(kind) = self.integration_point(tag_name) {
            self.current_element_ns = foreign_ns;

            return request_lexeme(move |this, lexeme| {
                expect_tag!(lexeme, StartTag { self_closing, .. } => {
                    if !self_closing {
                        this.push_frame(
                            ParseMode::Html,
                            foreign_ns,
                            FrameKind::IntegrationPoint(Box::new(IpState {
                                kind,
                                html_elements: Vec::new(),
                            })),
                        );
                    }
                    TreeBuilderFeedback::None
                })
            });
        }

        self.current_element_ns = foreign_ns;

        if tag_name == Tag::Font {
            // NOTE: `<font>` is a breakout only when it has one of these
            // attributes (13.2.6.5), so the lexeme is needed.
            return request_lexeme(move |this, lexeme| {
                expect_tag!(lexeme, StartTag { ref attributes, .. } => {
                    for attr in attributes.iter() {
                        let name = lexeme.part(attr.name);

                        if eq_case_insensitive(&name, b"color")
                            || eq_case_insensitive(&name, b"size")
                            || eq_case_insensitive(&name, b"face")
                        {
                            this.pop_to_html_content();
                            this.current_element_ns = Namespace::Html;
                            this.note_html_element(tag_name);

                            return TreeBuilderFeedback::None;
                        }
                    }
                });

                TreeBuilderFeedback::None
            });
        }

        if tag_name.is_empty() && self.mode() == ParseMode::MathML {
            // NOTE: tag name hash is empty - we need integration point check
            // for the possible <annotation-xml> case
            return request_lexeme(|this, lexeme| {
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
                                this.push_frame(
                                    ParseMode::Html,
                                    Namespace::MathML,
                                    FrameKind::IntegrationPoint(Box::new(IpState {
                                        kind: IntegrationPoint::AnnotationXml,
                                        html_elements: Vec::new(),
                                    })),
                                );

                                return TreeBuilderFeedback::None;
                            }
                        }
                    }
                });

                TreeBuilderFeedback::None
            });
        }

        // 13.2.6.5 "Any other start tag": a foreign element in the current
        // namespace, without switching the tokenizer. `<svg>`/`<math>` are no
        // exception here, but they still need a frame so that their own end tag
        // pops exactly them instead of an enclosing foreign element.
        if tag_name == Tag::Svg || tag_name == Tag::Math {
            let mode = self.mode();

            return request_lexeme(move |this, lexeme| {
                expect_tag!(lexeme, StartTag { self_closing, .. } => {
                    if !self_closing {
                        this.push_frame(mode, foreign_ns, FrameKind::Foreign(tag_name));
                    }
                    TreeBuilderFeedback::None
                })
            });
        }

        TreeBuilderFeedback::None
    }

    pub fn get_feedback_for_end_tag(&mut self, tag_name: LocalNameHash) -> TreeBuilderFeedback {
        if self.strict {
            self.ambiguity_guard.track_end_tag(tag_name);
        }

        if self.mode() == ParseMode::Html {
            self.end_tag_in_html_content(tag_name)
        } else {
            self.end_tag_in_foreign_content(tag_name)
        }
    }

    fn end_tag_in_foreign_content(&mut self, tag_name: LocalNameHash) -> TreeBuilderFeedback {
        // 13.2.6.5: `</p>` and `</br>` pop every foreign element up to the
        // nearest HTML element or integration point.
        if tag_is_one_of!(tag_name, [P, Br]) {
            self.pop_to_html_content();

            return TreeBuilderFeedback::None;
        }

        // The end tag of an open foreign element pops it, and every foreign
        // element opened after it. The walk stops at an integration point.
        for pos in (0..self.frames.len()).rev() {
            match self.frames[pos].kind {
                FrameKind::Foreign(open) if !tag_name.is_empty() && open == tag_name => {
                    self.frames.truncate(pos);

                    return TreeBuilderFeedback::None;
                }
                FrameKind::Foreign(_) => (),
                _ => break,
            }
        }

        TreeBuilderFeedback::None
    }

    fn end_tag_in_html_content(&mut self, tag_name: LocalNameHash) -> TreeBuilderFeedback {
        let frame_idx = self.frames.len() - 1;

        let Some(kind) = self.frames[frame_idx]
            .integration_point()
            .map(|state| state.kind)
        else {
            return TreeBuilderFeedback::None;
        };

        if tag_name.is_empty() {
            // NOTE: unhashed tag name - possibly <annotation-xml>
            return if matches!(kind, IntegrationPoint::AnnotationXml) {
                request_lexeme(|this, lexeme| {
                    expect_tag!(lexeme, EndTag { name, .. } => {
                        if eq_case_insensitive(&lexeme.part(name), b"annotation-xml") {
                            this.close_integration_point_if_current();
                        }
                        TreeBuilderFeedback::None
                    })
                })
            } else {
                TreeBuilderFeedback::None
            };
        }

        // 13.2.6.4.7 "any other end tag": walk the HTML elements opened inside
        // the integration point. The token pops the matching element and
        // everything above it, stops at an element in the special category,
        // and otherwise falls through to the integration point element itself
        // (which is special too, so only its own end tag closes it).
        let pops_through = pops_through_specials(tag_name);

        let mut matched = None;
        let mut blocked = false;

        if let Some(state) = self.frames[frame_idx].integration_point() {
            for pos in (0..state.html_elements.len()).rev() {
                let element = state.html_elements[pos];

                if element == tag_name {
                    matched = Some(pos);
                    break;
                }

                if !pops_through && is_special_html_element(element) {
                    blocked = true;
                    break;
                }
            }
        }

        if let Some(pos) = matched {
            if let FrameKind::IntegrationPoint(state) = &mut self.frames[frame_idx].kind {
                state.html_elements.truncate(pos);
            }
        } else if !blocked {
            let is_current = self.frames[frame_idx]
                .integration_point()
                .is_some_and(|state| state.html_elements.is_empty());

            if is_current && kind.element_name() == Some(tag_name) {
                self.frames.truncate(frame_idx);
            }
        }

        TreeBuilderFeedback::None
    }

    #[inline]
    fn push_frame(&mut self, mode: ParseMode, element_ns: Namespace, kind: FrameKind) {
        self.frames.push(Frame {
            mode,
            element_ns,
            kind,
        });
    }

    /// Pops foreign frames until the content is parsed with the HTML rules
    /// again (13.2.6.5: "pop elements from the stack of open elements" while the
    /// current node is not an HTML element or an integration point).
    fn pop_to_html_content(&mut self) {
        while self.frames.len() > 1 && self.mode() != ParseMode::Html {
            self.frames.pop();
        }
    }

    /// Records an HTML element opened inside an integration point, so that its
    /// end tag can be told apart from the integration point's own end tag.
    #[inline]
    fn note_html_element(&mut self, tag_name: LocalNameHash) {
        if is_void_tag(tag_name) {
            return;
        }

        if let Some(FrameKind::IntegrationPoint(state)) =
            self.frames.last_mut().map(|frame| &mut frame.kind)
        {
            state.html_elements.push(tag_name);
        }
    }

    #[inline]
    fn close_integration_point_if_current(&mut self) {
        let frame_idx = self.frames.len() - 1;

        let is_current = self.frames[frame_idx]
            .integration_point()
            .is_some_and(|state| state.html_elements.is_empty());

        if is_current {
            self.frames.truncate(frame_idx);
        }
    }

    #[inline]
    fn integration_point(&self, tag_name: LocalNameHash) -> Option<IntegrationPoint> {
        match self.mode() {
            ParseMode::Svg if is_html_integration_point_in_svg(tag_name) => {
                Some(IntegrationPoint::Svg(tag_name))
            }
            ParseMode::MathML if is_text_integration_point_in_math_ml(tag_name) => {
                Some(IntegrationPoint::MathMlText(tag_name))
            }
            _ => None,
        }
    }
}

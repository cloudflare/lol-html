//! There are few ambigious cases where we can't determine correct
//! parsing context having a limited information about the current
//! state of tree builder. This caused issues in the past where
//! Cloudflare's security features were used as XSS gadgets
//! (see https://portswigger.net/blog/when-security-features-collide).
//! Therefore, due to these safety concerns in such cases we prefer
//! to bail out from tokenization process.
//!
//! In tree builder simulation we need to switch tokenizer to one
//! of standalone text parsing state machines if we encounter some
//! specific tags. E.g. if we encounter `<script>` start tag we should
//! treat all content up to the closing `</script>` tag as text.
//! Without having a full-featured tree construction stage there is way
//! to trick tokenizer into parsing content that has actual tags in it
//! as text. E.g. by putting `<script>` start tag into context where
//! it will be ignored.
//!
//! There are just a few tree builder insertion modes in which text
//! parsing mode switching start tags can be ignored: in `<select>` and in
//! or after `<frameset>`.
//!
//! There are numerous not so obvious ways to get into or get out of these
//! insertion modes. So, for safety reasons we try to be pro-active here
//! and just bailout in case if we see text parsing mode switching start tags
//! between `<select>` start and end tag, or anywhere after the `<frameset>`
//! start tag. These cases shouldn't trigger bailout for any *conforming*
//! markup.
//!
//! However, there is a case where bailout could happen even with conforming
//! markup: if we encounter text parsing mode switching start tag in `<template>`
//! which is inside `<select>` element content. Unfortunately, rules required
//! to track template parsing context are way to complicated in such a case
//! and will require an implementation of the significant part of the tree
//! construction state. Though, current assumption is that markup that can
//! trigger this bailout case should be seen quite rarely in the wild.

use errors::TransformBailoutReason;
use tokenizer::TagName;

#[derive(Copy, Clone)]
enum TrackerState {
    Default,
    InSelect,
    InTemplateInSelect(u8),
    InOrAfterFrameset,
}

#[inline]
fn assert_not_ambigious_mode_switch(tag_name_hash: u64) -> Result<(), TransformBailoutReason> {
    if tag_is_one_of!(
        tag_name_hash,
        [Textarea, Title, Plaintext, Script, Style, Iframe, Xmp, Noembed, Noframes, Noscript]
    ) {
        Err(TransformBailoutReason::TextParsingAmbiguity)
    } else {
        Ok(())
    }
}

pub struct TextParsingAmbiguityTracker {
    state: TrackerState,
}

impl Default for TextParsingAmbiguityTracker {
    fn default() -> Self {
        TextParsingAmbiguityTracker {
            state: TrackerState::Default,
        }
    }
}

impl TextParsingAmbiguityTracker {
    pub fn track_start_tag(
        &mut self,
        tag_name_hash: Option<u64>,
    ) -> Result<(), TransformBailoutReason> {
        if let Some(t) = tag_name_hash {
            match self.state {
                TrackerState::Default => {
                    if t == TagName::Select {
                        self.state = TrackerState::InSelect;
                    } else if t == TagName::Frameset {
                        self.state = TrackerState::InOrAfterFrameset;
                    }
                }
                TrackerState::InSelect => {
                    // NOTE: these start tags cause premature exit
                    // from "in select" insertion mode.
                    if tag_is_one_of!(t, [Select, Textarea, Input, Keygen]) {
                        self.state = TrackerState::Default;
                    } else if t == TagName::Template {
                        self.state = TrackerState::InTemplateInSelect(1);
                    }
                    // NOTE: <script> is allowed in "in select" insertion mode.
                    else if t != TagName::Script {
                        assert_not_ambigious_mode_switch(t)?;
                    }
                }
                TrackerState::InTemplateInSelect(depth) => {
                    if t == TagName::Template {
                        if depth == u8::max_value() {
                            return Err(TransformBailoutReason::MaxTagNestingReached);
                        }

                        self.state = TrackerState::InTemplateInSelect(depth + 1);
                    } else {
                        assert_not_ambigious_mode_switch(t)?;
                    }
                }
                TrackerState::InOrAfterFrameset => {
                    // NOTE: <noframes> is allowed in and after <frameset>.
                    if t != TagName::Noframes {
                        assert_not_ambigious_mode_switch(t)?
                    }
                }
            }
        }

        Ok(())
    }

    pub fn track_end_tag(&mut self, tag_name_hash: Option<u64>) {
        if let Some(t) = tag_name_hash {
            match self.state {
                TrackerState::InSelect if t == TagName::Select => {
                    self.state = TrackerState::Default;
                }
                TrackerState::InTemplateInSelect(depth) if t == TagName::Template => {
                    self.state = if depth == 1 {
                        TrackerState::InSelect
                    } else {
                        TrackerState::InTemplateInSelect(depth - 1)
                    }
                }
                _ => (),
            }
        }
    }
}

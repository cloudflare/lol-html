use tag_name::TagName;
use tokenizer::TokenizerErrorKind;

#[derive(Copy, Clone)]
enum TrackerState {
    Default,
    InSelect,
    InOrAfterFrameset,
}

#[inline]
fn assert_not_ambigious_mode_switch(tag_name_hash: u64) -> Result<(), TokenizerErrorKind> {
    if tag_is_one_of!(
        tag_name_hash,
        [Textarea, Title, Plaintext, Script, Style, Iframe, Xmp, Noembed, Noframes, Noscript]
    ) {
        Err(TokenizerErrorKind::TextParsingAmbiguity)
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
    ) -> Result<(), TokenizerErrorKind> {
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
                    // NOTE: <select> start tag in "in select" insertion mode
                    // acts as an end tag. <textarea> being a text parsing mode
                    // switching tag causes premature closing of <select> as well.
                    // Both cases are conformance errors.
                    if t == TagName::Select || t == TagName::Textarea {
                        self.state = TrackerState::Default;
                    }
                    // NOTE: <script> is allowed in "in select" insertion mode.
                    else if t != TagName::Script {
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
        if let (Some(t), TrackerState::InSelect) = (tag_name_hash, self.state) {
            if t == TagName::Select {
                self.state = TrackerState::Default;
            }
        }
    }
}

use tag_name::TagName;
use tokenizer::TokenizerBailoutReason;

#[derive(Copy, Clone)]
enum TrackerState {
    Default,
    InSelect,
    InTemplateInSelect(u8),
    InOrAfterFrameset,
}

#[inline]
fn assert_not_ambigious_mode_switch(tag_name_hash: u64) -> Result<(), TokenizerBailoutReason> {
    if tag_is_one_of!(
        tag_name_hash,
        [Textarea, Title, Plaintext, Script, Style, Iframe, Xmp, Noembed, Noframes, Noscript]
    ) {
        Err(TokenizerBailoutReason::TextParsingAmbiguity)
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

// TODO template in select case
impl TextParsingAmbiguityTracker {
    pub fn track_start_tag(
        &mut self,
        tag_name_hash: Option<u64>,
    ) -> Result<(), TokenizerBailoutReason> {
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
                        let depth = depth + 1;

                        if depth == u8::max_value() {
                            return Err(TokenizerBailoutReason::MaxTagNestingReached);
                        }

                        self.state = TrackerState::InTemplateInSelect(depth);
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

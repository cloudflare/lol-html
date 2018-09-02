use super::{LexResultHandlerWithFeedback, Tokenizer};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TextParsingMode {
    Data,
    PlainText,
    RCData,
    RawText,
    ScriptData,
    CDataSection,
}

impl TextParsingMode {
    pub fn should_replace_unsafe_null_in_text(&self) -> bool {
        *self != TextParsingMode::Data && *self != TextParsingMode::CDataSection
    }

    pub fn allows_text_entitites(&self) -> bool {
        *self == TextParsingMode::Data || *self == TextParsingMode::RCData
    }
}

impl<'s> From<&'s str> for TextParsingMode {
    fn from(mode: &'s str) -> Self {
        match mode {
            "Data state" => TextParsingMode::Data,
            "PLAINTEXT state" => TextParsingMode::PlainText,
            "RCDATA state" => TextParsingMode::RCData,
            "RAWTEXT state" => TextParsingMode::RawText,
            "Script data state" => TextParsingMode::ScriptData,
            "CDATA section state" => TextParsingMode::CDataSection,
            _ => unreachable!("Unknown text parsing mode"),
        }
    }
}

impl<'t, H: LexResultHandlerWithFeedback> Into<fn(&mut Tokenizer<'t, H>, Option<u8>)>
    for TextParsingMode
{
    fn into(self) -> fn(&mut Tokenizer<'t, H>, Option<u8>) {
        match self {
            TextParsingMode::Data => Tokenizer::data_state,
            TextParsingMode::PlainText => Tokenizer::plaintext_state,
            TextParsingMode::RCData => Tokenizer::rcdata_state,
            TextParsingMode::RawText => Tokenizer::rawtext_state,
            TextParsingMode::ScriptData => Tokenizer::script_data_state,
            TextParsingMode::CDataSection => Tokenizer::cdata_section_state,
        }
    }
}

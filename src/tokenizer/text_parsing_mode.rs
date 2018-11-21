#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TextParsingMode {
    PlainText,
    RCData,
    RawText,
    ScriptData,
    Data,
    CDataSection,
}

#[cfg(feature = "testing_api")]
pub mod testing_api {
    use super::*;

    impl TextParsingMode {
        pub fn should_replace_unsafe_null_in_text(self) -> bool {
            self != TextParsingMode::Data && self != TextParsingMode::CDataSection
        }

        pub fn allows_text_entitites(self) -> bool {
            self == TextParsingMode::Data || self == TextParsingMode::RCData
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
                _ => panic!("Unknown text parsing mode"),
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub struct TextParsingModeSnapshot {
        pub mode: TextParsingMode,
        pub last_start_tag_name_hash: Option<u64>,
    }

    impl Default for TextParsingModeSnapshot {
        fn default() -> Self {
            TextParsingModeSnapshot {
                mode: TextParsingMode::Data,
                last_start_tag_name_hash: None,
            }
        }
    }

    declare_handler! {
        TextParsingModeChangeHandler(TextParsingModeSnapshot)
    }
}

#[cfg(feature = "testing_api")]
pub use self::testing_api::*;

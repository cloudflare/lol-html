use cfg_if::cfg_if;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TextParsingMode {
    PlainText,
    RCData,
    RawText,
    ScriptData,
    Data,
    CDataSection,
}

cfg_if! {
    if #[cfg(feature = "testing_api")] {
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

        pub trait TextParsingModeChangeHandler {
            fn handle(&mut self, mode: TextParsingMode);
        }

        impl<F: FnMut(TextParsingMode)> TextParsingModeChangeHandler for F {
            fn handle(&mut self, mode: TextParsingMode) {
                self(mode);
            }
        }
    }
}

use cfg_if::cfg_if;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TextType {
    PlainText,
    RCData,
    RawText,
    ScriptData,
    Data,
    CDataSection,
}

impl TextType {
    #[inline]
    pub fn allows_text_entitites(self) -> bool {
        self == TextType::Data || self == TextType::RCData
    }
}

cfg_if! {
    if #[cfg(feature = "test_api")] {
        impl TextType {
            pub fn should_replace_unsafe_null_in_text(self) -> bool {
                self != TextType::Data && self != TextType::CDataSection
            }
        }

        impl<'s> From<&'s str> for TextType {
            fn from(text_type: &'s str) -> Self {
                match text_type {
                    "Data state" => TextType::Data,
                    "PLAINTEXT state" => TextType::PlainText,
                    "RCDATA state" => TextType::RCData,
                    "RAWTEXT state" => TextType::RawText,
                    "Script data state" => TextType::ScriptData,
                    "CDATA section state" => TextType::CDataSection,
                    _ => panic!("Unknown text type"),
                }
            }
        }
    }
}

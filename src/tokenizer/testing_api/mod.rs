#[macro_use]
mod debug;

#[cfg(feature = "testing_api")]
#[derive(Copy, Clone)]
pub enum TextParsingMode {
    Data,
    PlainText,
    RCData,
    RawText,
    ScriptData,
    CDataSection,
}

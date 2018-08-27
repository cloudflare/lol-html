#[macro_use]
mod debug;

#[derive(Copy, Clone)]
pub enum TextParsingMode {
    Data,
    PlainText,
    RCData,
    RawText,
    ScriptData,
    CDataSection,
}

use crate::base::Bytes;

// TODO what to do with encodings when it's invalid in chunk boundaries
#[derive(Getters, Debug)]
pub struct Text<'i> {
    #[get = "pub"]
    text: Bytes<'i>,
    is_parsed: bool,
}

impl<'i> Text<'i> {
    pub(super) fn new_parsed(text: Bytes<'i>) -> Self {
        Text {
            text,
            is_parsed: true,
        }
    }

    #[inline]
    pub fn raw(&self) -> Option<&Bytes<'_>> {
        if self.is_parsed {
            Some(&self.text)
        } else {
            None
        }
    }
}

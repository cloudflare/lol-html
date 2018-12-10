use crate::base::Bytes;

#[derive(Getters, Debug)]
pub struct Text<'i> {
    #[get = "pub"]
    text: Bytes<'i>,
}

impl<'i> Text<'i> {
    pub(super) fn new_parsed(text: Bytes<'i>) -> Self {
        Text { text }
    }
}

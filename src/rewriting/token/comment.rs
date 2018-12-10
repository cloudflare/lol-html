use crate::base::Bytes;

#[derive(Getters, Debug)]
pub struct Comment<'i> {
    #[get = "pub"]
    text: Bytes<'i>,
}

impl<'i> Comment<'i> {
    pub(super) fn new_parsed(text: Bytes<'i>) -> Self {
        Comment { text }
    }
}

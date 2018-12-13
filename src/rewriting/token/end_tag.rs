use crate::base::Bytes;
use encoding_rs::Encoding;

#[derive(Getters, Debug)]
pub struct EndTag<'i> {
    #[get = "pub"]
    name: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
}

impl<'i> EndTag<'i> {
    pub(super) fn new_parsed(name: Bytes<'i>, raw: Bytes<'i>, encoding: &'static Encoding) -> Self {
        EndTag {
            name,
            raw: Some(raw),
            encoding,
        }
    }

    #[inline]
    pub fn raw(&self) -> Option<&Bytes<'i>> {
        self.raw.as_ref()
    }
}

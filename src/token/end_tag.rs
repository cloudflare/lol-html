use crate::base::Bytes;
use encoding_rs::Encoding;

#[derive(Debug)]
pub struct EndTag<'i> {
    name: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
}

impl<'i> EndTag<'i> {
    pub(crate) fn new_parsed(name: Bytes<'i>, raw: Bytes<'i>, encoding: &'static Encoding) -> Self {
        EndTag {
            name,
            raw: Some(raw),
            encoding,
        }
    }

    #[inline]
    pub fn name(&self) -> String {
        self.name.as_string(self.encoding)
    }

    #[inline]
    pub fn raw(&self) -> Option<&Bytes<'i>> {
        self.raw.as_ref()
    }
}

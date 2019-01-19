use crate::base::Bytes;
use encoding_rs::Encoding;

#[derive(Debug)]
pub struct EndTag<'i> {
    name: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
}

impl<'i> EndTag<'i> {
    pub(in crate::token) fn new_parsed(
        name: Bytes<'i>,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Self {
        EndTag {
            name,
            raw: Some(raw),
            encoding,
        }
    }

    #[inline]
    pub fn name(&self) -> String {
        let mut name = self.name.as_string(self.encoding);

        name.make_ascii_lowercase();

        name
    }
}

use crate::base::Bytes;
use encoding_rs::Encoding;

#[derive(Debug)]
pub struct Doctype<'i> {
    name: Option<Bytes<'i>>,
    public_id: Option<Bytes<'i>>,
    system_id: Option<Bytes<'i>>,
    force_quirks: bool,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
}

impl<'i> Doctype<'i> {
    pub(crate) fn new_parsed(
        name: Option<Bytes<'i>>,
        public_id: Option<Bytes<'i>>,
        system_id: Option<Bytes<'i>>,
        force_quirks: bool,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Self {
        Doctype {
            name,
            public_id,
            system_id,
            force_quirks,
            raw: Some(raw),
            encoding,
        }
    }

    #[inline]
    fn part_to_opt_str(&self, part: &Option<Bytes<'_>>) -> Option<String> {
        part.as_ref().map(|p| p.as_string(self.encoding))
    }

    #[inline]
    pub fn name(&self) -> Option<String> {
        self.part_to_opt_str(&self.name)
    }

    #[inline]
    pub fn public_id(&self) -> Option<String> {
        self.part_to_opt_str(&self.public_id)
    }

    #[inline]
    pub fn system_id(&self) -> Option<String> {
        self.part_to_opt_str(&self.system_id)
    }

    #[inline]
    pub fn force_quirks(&self) -> bool {
        self.force_quirks
    }

    #[inline]
    pub fn raw(&self) -> Option<&Bytes<'_>> {
        self.raw.as_ref()
    }
}

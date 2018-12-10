use crate::base::Bytes;

#[derive(Debug)]
pub struct Doctype<'i> {
    name: Option<Bytes<'i>>,
    public_id: Option<Bytes<'i>>,
    system_id: Option<Bytes<'i>>,
    force_quirks: bool,
}

impl<'i> Doctype<'i> {
    pub(super) fn new_parsed(
        name: Option<Bytes<'i>>,
        public_id: Option<Bytes<'i>>,
        system_id: Option<Bytes<'i>>,
        force_quirks: bool,
    ) -> Self {
        Doctype {
            name,
            public_id,
            system_id,
            force_quirks,
        }
    }

    #[inline]
    pub fn name(&self) -> Option<&Bytes<'i>> {
        self.name.as_ref()
    }

    #[inline]
    pub fn public_id(&self) -> Option<&Bytes<'i>> {
        self.public_id.as_ref()
    }

    #[inline]
    pub fn system_id(&self) -> Option<&Bytes<'i>> {
        self.system_id.as_ref()
    }

    #[inline]
    pub fn force_quirks(&self) -> bool {
        self.force_quirks
    }
}

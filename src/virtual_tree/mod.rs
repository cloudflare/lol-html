use crate::base::Bytes;

#[derive(Clone, PartialEq)]
pub enum LocalName<'b> {
    Hash(u64),
    Bytes(Bytes<'b>),
}

impl<'b> Eq for LocalName<'b> {}

// TODO encoding

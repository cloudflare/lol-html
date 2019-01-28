use crate::base::Bytes;

pub trait Serialize {
    fn raw(&self) -> Option<&Bytes<'_>>;
    fn serialize_from_parts(&self, output_handler: &mut dyn FnMut(&[u8]));

    #[inline]
    fn to_bytes(&self, output_handler: &mut dyn FnMut(&[u8])) {
        match self.raw() {
            Some(raw) => output_handler(raw),
            None => self.serialize_from_parts(output_handler),
        }
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    #[inline]
    fn raw(&self) -> Option<&Bytes<'_>> {
        None
    }

    #[inline]
    fn serialize_from_parts(&self, output_handler: &mut dyn FnMut(&[u8])) {
        for item in self {
            item.to_bytes(output_handler);
        }
    }
}

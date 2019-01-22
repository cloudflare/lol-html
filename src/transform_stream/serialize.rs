use crate::base::Bytes;
use std::borrow::Borrow;

pub trait Serialize {
    fn raw(&self) -> Option<&Bytes<'_>>;
    fn serialize_from_parts(&self, handler: &mut dyn FnMut(&Bytes<'_>));

    #[inline]
    fn to_bytes(&self, handler: &mut dyn FnMut(&Bytes<'_>)) {
        match self.raw() {
            Some(raw) => handler(raw),
            None => self.serialize_from_parts(handler),
        }
    }
}

impl<T: Serialize> Serialize for Borrow<[T]> {
    #[inline]
    fn raw(&self) -> Option<&Bytes<'_>> {
        None
    }

    #[inline]
    fn serialize_from_parts(&self, handler: &mut dyn FnMut(&Bytes<'_>)) {
        for item in self.borrow().iter() {
            item.to_bytes(handler);
        }
    }
}

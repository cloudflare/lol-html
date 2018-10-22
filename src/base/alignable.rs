pub trait Alignable {
    fn align(&mut self, offset: usize);
}

impl<T: Alignable> Alignable for Vec<T> {
    #[inline]
    fn align(&mut self, offset: usize) {
        for item in self.iter_mut() {
            item.align(offset);
        }
    }
}

impl<T: Alignable> Alignable for Option<T> {
    #[inline]
    fn align(&mut self, offset: usize) {
        if let Some(val) = self {
            val.align(offset);
        }
    }
}

impl Alignable for usize {
    #[inline]
    fn align(&mut self, offset: usize) {
        *self -= offset;
    }
}

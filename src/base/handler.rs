macro_rules! declare_handler {
    ($name:ident, $ty:ty) => {
        pub trait $name {
            fn handle(&mut self, val: &$ty);
        }

        impl<F: FnMut(&$ty)> $name for F {
            fn handle(&mut self, val: &$ty) {
                self(val);
            }
        }
    };
}

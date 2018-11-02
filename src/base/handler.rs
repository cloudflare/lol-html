macro_rules! declare_handler {
    ( $name:ident($ty:ty) $(-> $ret_ty:ty)* ) => {
        pub trait $name {
            fn handle(&mut self, val: &$ty) $(-> $ret_ty)*;
        }

        impl<F: FnMut(&$ty) $(-> $ret_ty)*> $name for F {
            fn handle(&mut self, val: &$ty) $(-> $ret_ty)* {
                self(val)
            }
        }
    };
}

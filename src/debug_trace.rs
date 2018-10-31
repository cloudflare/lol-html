macro_rules! trace {
    ( @actions $($actions:tt)+ ) => {
        #[cfg(feature = "debug_trace")]
        println!("@action: {}", stringify!($($actions)+));
    };

    ( @chars $action_descr:expr $(, $ch:expr)* ) => {
        #[cfg(feature = "debug_trace")]
        {
            print!(">{}", $action_descr);

            $({
                use std::char;

                print!(": {:?}", $ch.map(|ch| unsafe { char::from_u32_unchecked(ch as u32) }));
            })*

            println!();
        }
    };

    ( @buffer $buffer:expr ) => {
        #[cfg(feature = "debug_trace")]
        {
            use base::Bytes;

            println!("-- Buffered: {:#?}", Bytes::from($buffer.bytes()));
        }
    };

    ( @write $slice:expr ) => {
        #[cfg(feature = "debug_trace")]
        {
            use base::Bytes;

            println!("-- Write: {:#?}", Bytes::from($slice));
        }
    };

    ( @end ) => {
        #[cfg(feature = "debug_trace")]
        {
            println!("-- End");
        }
    };

    ( @chunk $chunk:expr ) => {
        #[cfg(feature = "debug_trace")]
        {
            println!();
            println!("{:#?}", $chunk);
            println!();
        }
    };
}
